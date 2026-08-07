use super::*;
use crate::shader_const::Layout;
use kiss3d::prelude::*;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, ComputePipeline,
    ComputePipelineDescriptor, PipelineLayoutDescriptor, ShaderStages,
};

/// Define connections between outputs and inputs
pub const BIND_CONNECTIONS: Layout = Layout::new(0, 0);

/// Define the type of operation and the number of inputs to read for each output
pub const BIND_OPERATIONS: Layout = Layout::new(0, 1);

/// Input values to process
pub const BIND_INPUTS: Layout = Layout::new(1, 0);

/// Ouput values to write
pub const BIND_OUTPUTS: Layout = Layout::new(1, 1);

impl ComputeLayout {
    /// Create the layouts for the compute shader
    pub fn new(ctx: &Context) -> Self {
        Self {
            layout_const: create_const_layout(ctx),
            layout_mut: create_mut_layout(ctx),
        }
    }
}

/// Load the compute shader and build the pipeline used to run one simulation tick
pub fn create_compute_pipeline(ctx: &Context, layout: &ComputeLayout) -> ComputePipeline {
    let shader = ctx.create_shader_module(
        Some("logic_compute_shader"),
        include_str!("../../shaders/compute.wgsl"),
    );
    let pipeline_layout = ctx.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("logic_compute_pipeline_layout"),
        bind_group_layouts: &[Some(&layout.layout_const), Some(&layout.layout_mut)],
        immediate_size: 0,
    });
    ctx.device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("logic_compute_pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("process"),
        compilation_options: Default::default(),
        cache: None,
    })
}

impl ComputeBinding {
    /// Create the binding based on the buffer and layout that have been generated
    pub fn new(ctx: &Context, layout: &ComputeLayout, buffers: &AllocBuffers) -> Self {
        let [ping, pong] = &buffers.ping_pong;
        Self {
            bind_const: create_const_bind(
                ctx,
                &layout.layout_const,
                &buffers.connections,
                &buffers.operations,
            ),
            ping_pong: [
                create_mut_bind(ctx, "ping", &layout.layout_mut, ping, pong),
                create_mut_bind(ctx, "pong", &layout.layout_mut, pong, ping),
            ],
        }
    }
}

/// Create a bind group layout for the two constant buffers `connections` and `operations`
fn create_const_layout(ctx: &Context) -> BindGroupLayout {
    ctx.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("constants"),
        entries: &[
            BindGroupLayoutEntry {
                binding: BIND_CONNECTIONS.binding,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: BIND_OPERATIONS.binding,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

/// Create a bind group layout for the two mutable ping-pong buffers
fn create_mut_layout(ctx: &Context) -> BindGroupLayout {
    ctx.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("mutables"),
        entries: &[
            BindGroupLayoutEntry {
                // Only the compute pass ever binds these buffers with write
                // access; `VERTEX_FRAGMENT` here would require the
                // `VERTEX_WRITABLE_STORAGE` feature for no reason (the
                // render pass reads the same buffer through the separate,
                // read-only `processed` bind group instead).
                //
                // `compute.wgsl` declares `inputs` as `read` only (only
                // `outputs` is `read_write`); the layout must match exactly
                // or pipeline creation fails validation.
                binding: BIND_INPUTS.binding,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: BIND_OUTPUTS.binding,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

/// Create a bind group for the constant storage buffers
fn create_const_bind(
    ctx: &Context,
    layout: &BindGroupLayout,
    connections: &Buffer,
    operations: &Buffer,
) -> BindGroup {
    ctx.create_bind_group(&BindGroupDescriptor {
        label: Some("constants"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: BIND_CONNECTIONS.binding,
                resource: connections.as_entire_binding(),
            },
            BindGroupEntry {
                binding: BIND_OPERATIONS.binding,
                resource: operations.as_entire_binding(),
            },
        ],
    })
}

/// Create a bind group for the mutable ping-pong buffers
fn create_mut_bind(
    ctx: &Context,
    label: &str,
    layout: &BindGroupLayout,
    inputs: &Buffer,
    outputs: &Buffer,
) -> BindGroup {
    ctx.create_bind_group(&BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            BindGroupEntry {
                binding: BIND_INPUTS.binding,
                resource: inputs.as_entire_binding(),
            },
            BindGroupEntry {
                binding: BIND_OUTPUTS.binding,
                resource: outputs.as_entire_binding(),
            },
        ],
    })
}
