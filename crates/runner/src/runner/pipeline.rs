use super::{AllocBuffers, ComputeBinding, ComputeLayout, RenderBinding, RenderLayout};
use crate::{
    loader::{Circuit, converter::MeshDraw},
    shader_const::{Mat4x4, Vertex, WORKGROUP},
};
use wgpu::{
    ColorTargetState, ColorWrites, CommandEncoder, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, DepthStencilState, Device, FragmentState, IndexFormat,
    MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, TextureFormat, TextureView, VertexBufferLayout,
    VertexState, VertexStepMode, vertex_attr_array,
};

/// Vertex attributes matching `ToVertex` in `render.wgsl`
const VERTEX_ATTRS: [wgpu::VertexAttribute; 2] = vertex_attr_array![0 => Float32x3, 1 => Float32x3];

/// Owns the GPU resources for a circuit and drives its simulation and rendering.
///
/// Rendering batches every instance sharing the same mesh into a single
/// `draw_indexed` call, so the number of draw calls only grows with the
/// number of distinct component shapes, not with the number of components.
pub struct Runner {
    alloc: AllocBuffers,
    compute_binding: ComputeBinding,
    render_binding: RenderBinding,
    compute_pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
    draws: Vec<MeshDraw>,
    component_count: u32,

    /// Index of the ping-pong configuration whose output holds the latest
    /// simulation state. `step` reads this to know which buffers to use;
    /// `advance` flips it once the current state has been rendered.
    current: usize,
}

impl Runner {
    /// Upload a circuit to the GPU and build the pipelines needed to run and render it
    pub fn new(
        device: &Device,
        circuit: &Circuit,
        projection: &Mat4x4,
        color_format: TextureFormat,
        depth_format: Option<TextureFormat>,
    ) -> Self {
        let processing = circuit.processing();
        let rendering = circuit.rendering();

        let alloc = AllocBuffers::new(
            device,
            projection,
            processing.connections(),
            processing.operations(),
            rendering.vertices(),
            rendering.indices(),
            rendering.instances(),
            rendering.color_presets(),
            processing.initial_values(),
        );

        let compute_layout = ComputeLayout::new(device);
        let compute_binding = ComputeBinding::new(device, &compute_layout, &alloc);

        let render_layout = RenderLayout::new(device);
        let render_binding = RenderBinding::new(device, &render_layout, &alloc);

        let compute_pipeline = create_compute_pipeline(device, &compute_layout);
        let render_pipeline =
            create_render_pipeline(device, &render_layout, color_format, depth_format);

        Self {
            alloc,
            compute_binding,
            render_binding,
            compute_pipeline,
            render_pipeline,
            draws: rendering.draws().to_vec(),
            component_count: processing.operations().len() as u32,
            current: 0,
        }
    }

    /// Run one simulation tick. Call `render` before the next `step`
    /// to display the state it produces, then `advance`.
    pub fn step(&self, encoder: &mut CommandEncoder) {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_pipeline(&self.compute_pipeline);
        pass.set_bind_group(0, &self.compute_binding.bind_const, &[]);
        pass.set_bind_group(1, &self.compute_binding.ping_pong[self.current], &[]);
        pass.dispatch_workgroups(self.component_count.div_ceil(WORKGROUP), 1, 1);
    }

    /// Render the state produced by the last `step`, batching all instances
    /// of each mesh into a single `draw_indexed` call.
    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        color_view: &TextureView,
        depth_view: Option<&TextureView>,
    ) {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: depth_view.map(|view| RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.render_pipeline);
        pass.set_bind_group(0, &self.render_binding.bind_proj, &[]);
        pass.set_bind_group(1, &self.render_binding.bind_static, &[]);
        pass.set_bind_group(2, &self.render_binding.ping_pong[self.current], &[]);
        pass.set_vertex_buffer(0, self.alloc.vertices.slice(..));
        pass.set_index_buffer(self.alloc.indices.slice(..), IndexFormat::Uint32);

        for draw in &self.draws {
            pass.draw_indexed(
                draw.indices.clone(),
                draw.base_vertex,
                draw.instances.clone(),
            );
        }
    }

    /// Swap the ping-pong buffers, must be called once per tick after `render`
    /// so the next `step` reads the state that was just displayed.
    #[inline]
    pub fn advance(&mut self) {
        self.current ^= 1;
    }

    /// Read back the value computed for every component, in IO order.
    /// This blocks until the GPU is done; meant for debugging and testing,
    /// not for use in a per-frame render loop.
    pub fn read_processed(&self, device: &Device, queue: &Queue) -> Vec<u32> {
        let source = &self.alloc.ping_pong[self.current];
        let size = source.size();

        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_buffer_to_buffer(source, 0, &staging, 0, size);
        queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .expect("failed to poll the device while waiting for the buffer to be mapped");

        let result = {
            let data = slice.get_mapped_range();
            bytemuck::cast_slice::<u8, u32>(&data).to_vec()
        };
        staging.unmap();
        result
    }
}

/// Build the compute pipeline running `compute.wgsl`
fn create_compute_pipeline(device: &Device, layout: &ComputeLayout) -> ComputePipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/compute.wgsl"));

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("compute"),
        bind_group_layouts: &[Some(&layout.layout_const), Some(&layout.layout_mut)],
        immediate_size: 0,
    });

    device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("compute"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("process"),
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    })
}

/// Build the render pipeline running `render.wgsl`
fn create_render_pipeline(
    device: &Device,
    layout: &RenderLayout,
    color_format: TextureFormat,
    depth_format: Option<TextureFormat>,
) -> RenderPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("../../shaders/render.wgsl"));

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("render"),
        bind_group_layouts: &[
            Some(&layout.layout_proj),
            Some(&layout.layout_static),
            Some(&layout.layout_process),
        ],
        immediate_size: 0,
    });

    let vertex_buffer = VertexBufferLayout {
        array_stride: size_of::<Vertex>() as u64,
        step_mode: VertexStepMode::Vertex,
        attributes: &VERTEX_ATTRS,
    };

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("render"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vertex_main"),
            compilation_options: PipelineCompilationOptions::default(),
            buffers: &[vertex_buffer],
        },
        primitive: PrimitiveState::default(),
        depth_stencil: depth_format.map(|format| DepthStencilState {
            format,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fragment_main"),
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(ColorTargetState {
                format: color_format,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
