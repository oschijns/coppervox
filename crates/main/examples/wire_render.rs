//! Build a tiny synthetic voxel model in memory, convert it into a logic
//! circuit and a batched GPU mesh, run one simulation tick and render it
//! off-screen. This proves the conversion pipeline (labeling -> meshing ->
//! mesh batching -> GPU buffers) and the color-from-value render path work,
//! without needing a window or any model/config files on disk.
//!
//! Layout (single row along X):
//! - `(0,0,0)`             : a constant power source (always high)
//! - `(1,0,0)..(3,0,0)`    : "wire A", a 3-voxel run merged into one component,
//!                           touching the source, so it ends up driven high
//! - `(10,0,0)..(12,0,0)`  : "wire B", the same shape as wire A but not
//!                           touching anything, so it stays low
//!
//! Wire A and wire B are disconnected but share the exact same local shape,
//! so they should be batched into a single mesh / single draw call, while
//! ending up with different colors once the simulation is stepped.

use coppervox_common::{Model, gridmap::GridMap};
use coppervox_config::{
    Config,
    color::ColorSet,
    component::{CompDef, CompType},
};
use coppervox_runner::{loader::Circuit, runner::Runner, shader_const::Mat4x4};
use kiss3d::color::Rgba;
use std::collections::HashMap;

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

/// Column-major orthographic projection, matching WGSL's `mat4x4<f32>` layout
fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4x4 {
    let sx = 2.0 / (right - left);
    let sy = 2.0 / (top - bottom);
    let sz = 1.0 / (far - near);
    [
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, sz, 0.0],
        [
            -(right + left) / (right - left),
            -(top + bottom) / (top - bottom),
            -near / (far - near),
            1.0,
        ],
    ]
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

    // Set up a headless GPU device: no window or surface needed to prove the
    // conversion pipeline produces valid, renderable GPU data.
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("no compatible GPU adapter found");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .expect("failed to open device");

    const SIZE: u32 = 64;
    let color_format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let depth_format = wgpu::TextureFormat::Depth32Float;

    let extent = wgpu::Extent3d {
        width: SIZE,
        height: SIZE,
        depth_or_array_layers: 1,
    };
    let color_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("color"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: color_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let color_view = color_texture.create_view(&Default::default());

    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: depth_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth_texture.create_view(&Default::default());

    let projection = orthographic(-1.0, 14.0, -2.0, 2.0, -10.0, 10.0);
    let mut runner = Runner::new(&device, &circuit, &projection, color_format, Some(depth_format));

    // Advance the simulation by one tick and render the resulting state.
    // This exercises the full compute + render pipeline: if any buffer or
    // bind group were wired up incorrectly, wgpu validation would panic here.
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    runner.step(&mut encoder);
    runner.render(&mut encoder, &color_view, Some(&depth_view));
    queue.submit(Some(encoder.finish()));
    runner.advance();

    let processed = runner.read_processed(&device, &queue);
    println!("processed values (IO order): {processed:?}");
    println!(
        "wire A (connected to source) and wire B (disconnected) share a mesh \
         but should differ in value: source propagates {:#010x} into wire A, \
         wire B stays at 0.",
        u32::MAX
    );
}
