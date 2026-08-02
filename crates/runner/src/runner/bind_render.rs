use super::*;
use crate::shader_const::{Layout, Mat4x4};
use std::num::NonZero;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, Device, ShaderStages,
};

/// The singular projection matrix of the camera
pub const BIND_PROJECTION: Layout = Layout::new(0, 0);

/// Per-instance data (transform, colors index, process index)
pub const BIND_INSTANCES: Layout = Layout::new(1, 0);

/// Color presets
pub const BIND_COLOR_PRESETS: Layout = Layout::new(1, 1);

/// Processed values to display on the components
pub const BIND_PROCESSED: Layout = Layout::new(2, 0);

impl RenderLayout {
    /// Create the layouts for the render shader
    pub fn new(device: &Device) -> Self {
        Self {
            layout_proj: create_proj_layout(device),
            layout_static: create_static_layout(device),
            layout_process: create_process_layout(device),
        }
    }
}

impl RenderBinding {
    /// Create the binding based on the buffer and layout that have been generated
    pub fn new(device: &Device, layout: &RenderLayout, buffers: &AllocBuffers) -> Self {
        let [ping, pong] = &buffers.ping_pong;
        Self {
            bind_proj: create_proj_bind(device, &layout.layout_proj, &buffers.projection),
            bind_static: create_static_bind(
                device,
                &layout.layout_static,
                &buffers.instances,
                &buffers.color_presets,
            ),
            ping_pong: [
                create_process_bind(device, "ping", &layout.layout_process, pong),
                create_process_bind(device, "pong", &layout.layout_process, ping),
            ],
        }
    }
}

/// Create a bind group layout for the projection matrix
fn create_proj_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("projection"),
        entries: &[BindGroupLayoutEntry {
            binding: BIND_PROJECTION.binding,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZero::new(size_of::<Mat4x4>() as u64),
            },
            count: None,
        }],
    })
}

/// Create a bind group layout for the instances and color presets storage buffers
fn create_static_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("statics"),
        entries: &[
            BindGroupLayoutEntry {
                binding: BIND_INSTANCES.binding,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: BIND_COLOR_PRESETS.binding,
                visibility: ShaderStages::VERTEX,
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
fn create_process_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("processed"),
        entries: &[BindGroupLayoutEntry {
            binding: BIND_PROCESSED.binding,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

/// Create a bind group for the projection matrix
fn create_proj_bind(device: &Device, layout: &BindGroupLayout, projection: &Buffer) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("projection"),
        layout,
        entries: &[BindGroupEntry {
            binding: BIND_PROJECTION.binding,
            resource: projection.as_entire_binding(),
        }],
    })
}

/// Create a bind group for the instances and color presets storage buffers
fn create_static_bind(
    device: &Device,
    layout: &BindGroupLayout,
    instances: &Buffer,
    colors: &Buffer,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("statics"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: BIND_INSTANCES.binding,
                resource: instances.as_entire_binding(),
            },
            BindGroupEntry {
                binding: BIND_COLOR_PRESETS.binding,
                resource: colors.as_entire_binding(),
            },
        ],
    })
}

/// Create a bind group for the processed ping-pong buffers
fn create_process_bind(
    device: &Device,
    label: &str,
    layout: &BindGroupLayout,
    buffer: &Buffer,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[BindGroupEntry {
            binding: BIND_PROCESSED.binding,
            resource: buffer.as_entire_binding(),
        }],
    })
}
