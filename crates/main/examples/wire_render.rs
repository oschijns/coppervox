//! Build a tiny synthetic voxel model in memory, convert it into a logic
//! circuit and a batched GPU mesh, and display it live in a window: every
//! frame steps the compute simulation and renders the result. This proves
//! the conversion pipeline (labeling -> meshing -> mesh batching -> GPU
//! buffers) and the color-from-value render path work, without needing any
//! model/config files on disk.
//!
//! Layout (single row along X):
//! - `(0,0,0)`             : a constant power source (always high)
//! - `(1,0,0)..(3,0,0)`    : "wire A", a 3-voxel run merged into one component,
//!                           touching the source, so it ends up driven high
//! - `(10,0,0)..(12,0,0)`  : "wire B", the same shape as wire A but not
//!                           touching anything, so it stays low
//!
//! Wire A and wire B are disconnected but share the exact same local shape,
//! so they are batched into a single mesh / single draw call, while ending
//! up with different colors once the simulation is stepped - the source
//! (red cube) and wire A (cyan bar) should read bright, wire B (also a cyan
//! bar, sharing wire A's draw call) should stay dark.

use coppervox_common::{Model, gridmap::GridMap};
use coppervox_config::{
    Config,
    color::ColorSet,
    component::{CompDef, CompType},
};
use coppervox_runner::{loader::Circuit, runner::Runner, shader_const::Mat4x4};
use kiss3d::color::Rgba;
use std::{collections::HashMap, sync::Arc};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const VOXEL_SOURCE: u32 = 1;
const VOXEL_WIRE: u32 = 2;

fn build_model() -> Model {
    let mut model: Model = GridMap::new([16, 16, 16]);

    model.set::<usize>(&[0, 0, 0], VOXEL_SOURCE);

    model.set::<usize>(&[1, 0, 0], VOXEL_WIRE);
    model.set::<usize>(&[2, 0, 0], VOXEL_WIRE);
    model.set::<usize>(&[3, 0, 0], VOXEL_WIRE);

    model.set::<usize>(&[10, 0, 0], VOXEL_WIRE);
    model.set::<usize>(&[11, 0, 0], VOXEL_WIRE);
    model.set::<usize>(&[12, 0, 0], VOXEL_WIRE);

    model
}

fn build_config() -> Config {
    let mut map = HashMap::new();
    map.insert(
        VOXEL_SOURCE,
        CompDef {
            comp_type: CompType::Const(u32::MAX),
            color_index: 0,
        },
    );
    map.insert(
        VOXEL_WIRE,
        CompDef {
            comp_type: CompType::Or,
            color_index: 1,
        },
    );

    let colors = vec![
        // source: dim red when off, bright yellow when high
        ColorSet {
            off: Rgba::new(20, 0, 0, 255),
            on_low: Rgba::new(120, 60, 0, 255),
            on_high: Rgba::new(255, 220, 40, 255),
        },
        // wire: dark blue when off, bright cyan when high
        ColorSet {
            off: Rgba::new(10, 10, 30, 255),
            on_low: Rgba::new(40, 80, 160, 255),
            on_high: Rgba::new(80, 220, 255, 255),
        },
    ];

    Config { map, colors }
}

/// Column-major matrix multiply, matching WGSL's `mat4x4<f32>` layout
fn mat4_mul(a: Mat4x4, b: Mat4x4) -> Mat4x4 {
    let mut out = [[0.0f32; 4]; 4];
    for col in 0..4 {
        for row in 0..4 {
            out[col][row] = (0..4).map(|k| a[k][row] * b[col][k]).sum();
        }
    }
    out
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(a: [f32; 3]) -> [f32; 3] {
    let len = dot(a, a).sqrt();
    [a[0] / len, a[1] / len, a[2] / len]
}

/// Right-handed view matrix, matching WGSL's `mat4x4<f32>` column-major layout
fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Mat4x4 {
    let f = normalize(sub(target, eye));
    let s = normalize(cross(f, up));
    let u = cross(s, f);
    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0],
    ]
}

/// Right-handed perspective projection with a 0..1 depth range, matching wgpu's clip space
fn perspective(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Mat4x4 {
    let cot = 1.0 / (fovy_radians * 0.5).tan();
    [
        [cot / aspect, 0.0, 0.0, 0.0],
        [0.0, cot, 0.0, 0.0],
        [0.0, 0.0, far / (near - far), -1.0],
        [0.0, 0.0, (near * far) / (near - far), 0.0],
    ]
}

fn camera(aspect: f32) -> Mat4x4 {
    // Look at the middle of the scene from an angle, so the cubes read as 3D.
    let view = look_at([6.0, 6.0, 12.0], [6.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let proj = perspective(45f32.to_radians(), aspect, 0.1, 100.0);
    mat4_mul(proj, view)
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

fn create_depth_view(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&Default::default())
}

/// GPU resources that can only be created once we have a window (and, on
/// some platforms, only after the app has been `resumed`).
struct Gpu {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    depth_view: wgpu::TextureView,
    runner: Runner,
}

struct App {
    circuit: Circuit,
    gpu: Option<Gpu>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("CopperVox - wire_render example")
            .with_inner_size(LogicalSize::new(800.0, 600.0));
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("failed to create window"),
        );

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("no compatible GPU adapter found");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("failed to open device");

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let depth_view = create_depth_view(&device, surface_config.width, surface_config.height);
        let aspect = surface_config.width as f32 / surface_config.height as f32;
        let projection = camera(aspect);

        let runner = Runner::new(
            &device,
            &self.circuit,
            &projection,
            surface_format,
            Some(DEPTH_FORMAT),
        );

        window.request_redraw();
        self.gpu = Some(Gpu {
            window,
            device,
            queue,
            surface,
            surface_config,
            depth_view,
            runner,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gpu) = &mut self.gpu else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                gpu.surface_config.width = size.width.max(1);
                gpu.surface_config.height = size.height.max(1);
                gpu.surface.configure(&gpu.device, &gpu.surface_config);
                gpu.depth_view =
                    create_depth_view(&gpu.device, gpu.surface_config.width, gpu.surface_config.height);
            }

            WindowEvent::RedrawRequested => {
                let frame = match gpu.surface.get_current_texture() {
                    Ok(frame) => frame,
                    // The surface can go stale on resize/minimize; reconfigure and retry next frame.
                    Err(_) => {
                        gpu.surface.configure(&gpu.device, &gpu.surface_config);
                        gpu.window.request_redraw();
                        return;
                    }
                };
                let view = frame.texture.create_view(&Default::default());

                let mut encoder = gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                gpu.runner.step(&mut encoder);
                gpu.runner.render(&mut encoder, &view, Some(&gpu.depth_view));
                gpu.queue.submit(Some(encoder.finish()));
                frame.present();
                gpu.runner.advance();

                gpu.window.request_redraw();
            }

            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let model = build_model();
    let config = build_config();
    let circuit = Circuit::make(&model, &config).expect("failed to build circuit");

    let rendering = circuit.rendering();
    println!(
        "components: {} | vertices: {} | indices: {} | draw calls: {}",
        rendering.instances().len(),
        rendering.vertices().len(),
        rendering.indices().len(),
        rendering.draws().len(),
    );
    for (i, draw) in rendering.draws().iter().enumerate() {
        println!(
            "  draw {i}: mesh indices {}..{}, base_vertex {}, instances {}..{}",
            draw.indices.start, draw.indices.end, draw.base_vertex, draw.instances.start,
            draw.instances.end,
        );
    }

    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        circuit,
        gpu: None,
    };
    event_loop.run_app(&mut app).expect("event loop failed");
}
