use super::*;
use crate::shader_const::{BIND_CONNECTIONS, BIND_INPUTS, BIND_OPERATIONS, BIND_OUTPUTS};
use kiss3d::prelude::*;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, ShaderStages,
};

impl ComputeLayout {
    /// Create the layouts for the compute shader
    pub fn new(ctx: &Context) -> Self {
        Self {
            layout_const: create_const_layout(ctx),
            layout_mut: create_mut_layout(ctx),
        }
    }
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
                binding: BIND_INPUTS.binding,
                visibility: ShaderStages::COMPUTE | ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: BIND_OUTPUTS.binding,
                visibility: ShaderStages::COMPUTE | ShaderStages::VERTEX_FRAGMENT,
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
