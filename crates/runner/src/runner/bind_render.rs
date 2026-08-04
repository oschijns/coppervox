use super::*;
use crate::shader_const::{Layout, Mat4x4};
use kiss3d::prelude::*;
use std::num::NonZero;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, ShaderStages,
};

/// The singular projection matrix of the camera
pub const BIND_PROJECTION: Layout = Layout::new(0, 0);

/// Instance data (translation & color index) of the components
pub const BIND_INSTANCES_DATA: Layout = Layout::new(1, 0);

/// Color presets
pub const BIND_COLOR_PRESETS: Layout = Layout::new(1, 1);

/// Processed values to display on the components
pub const BIND_PROCESSED: Layout = Layout::new(2, 0);

impl RenderLayout {
    /// Create the layouts for the compute shader
    pub fn new(ctx: &Context) -> Self {
        Self {
            layout_proj: create_proj_layout(ctx),
            layout_static: create_static_layout(ctx),
            layout_process: create_process_layout(ctx),
        }
    }
}

impl RenderBinding {
    /// Create the binding based on the buffer and layout that have been generated
    pub fn new(ctx: &Context, layout: &RenderLayout, buffers: &AllocBuffers) -> Self {
        let [ping, pong] = &buffers.ping_pong;
        Self {
            bind_proj: create_proj_bind(ctx, &layout.layout_proj, &buffers.projection),
            bind_static: create_static_bind(
                ctx,
                &layout.layout_static,
                &buffers.instances_data,
                &buffers.color_presets,
            ),
            ping_pong: [
                create_process_bind(ctx, "ping", &layout.layout_process, pong),
                create_process_bind(ctx, "pong", &layout.layout_process, ping),
            ],
        }
    }
}

/// Create a bind group layout for the two constant buffers `connections` and `operations`
fn create_proj_layout(ctx: &Context) -> BindGroupLayout {
    ctx.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("projection"),
        entries: &[BindGroupLayoutEntry {
            binding: BIND_PROJECTION.binding,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZero::new(size_of::<Mat4x4>() as u64),
            },
            count: None,
        }],
    })
}

/// Create a bind group layout for the two constant buffers `connections` and `operations`
fn create_static_layout(ctx: &Context) -> BindGroupLayout {
    ctx.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("statics"),
        entries: &[
            BindGroupLayoutEntry {
                binding: BIND_INSTANCES_DATA.binding,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: BIND_COLOR_PRESETS.binding,
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

/// Create a bind group layout for the two processed ping-pong buffers
fn create_process_layout(ctx: &Context) -> BindGroupLayout {
    ctx.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("processed"),
        entries: &[BindGroupLayoutEntry {
            binding: BIND_PROCESSED.binding,
            visibility: ShaderStages::COMPUTE | ShaderStages::VERTEX_FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

/// Create a bind group for the constant storage buffers
fn create_proj_bind(ctx: &Context, layout: &BindGroupLayout, projection: &Buffer) -> BindGroup {
    ctx.create_bind_group(&BindGroupDescriptor {
        label: Some("projection"),
        layout,
        entries: &[BindGroupEntry {
            binding: BIND_PROJECTION.binding,
            resource: projection.as_entire_binding(),
        }],
    })
}

/// Create a bind group for the static storage buffers containing
fn create_static_bind(
    ctx: &Context,
    layout: &BindGroupLayout,
    instances_data: &Buffer,
    color_presets: &Buffer,
) -> BindGroup {
    ctx.create_bind_group(&BindGroupDescriptor {
        label: Some("statics"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: BIND_INSTANCES_DATA.binding,
                resource: instances_data.as_entire_binding(),
            },
            BindGroupEntry {
                binding: BIND_COLOR_PRESETS.binding,
                resource: color_presets.as_entire_binding(),
            },
        ],
    })
}

/// Create a bind group for the processed ping-pong buffers
fn create_process_bind(
    ctx: &Context,
    label: &str,
    layout: &BindGroupLayout,
    buffer: &Buffer,
) -> BindGroup {
    ctx.create_bind_group(&BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[BindGroupEntry {
            binding: BIND_PROCESSED.binding,
            resource: buffer.as_entire_binding(),
        }],
    })
}
