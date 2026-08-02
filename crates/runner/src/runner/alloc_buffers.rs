/*
 * Since modern CPU and GPU both works in little endian,
 * we do not need to swap endianness when submitting buffers to the CPU.
 * Either way Vulkan require CPU and GPU to share the same endianness.
 */

use super::*;
use crate::shader_const::{ColorPreset, InstanceData, Mat4x4, Vertex};
use wgpu::{BufferUsages, Device, util::DeviceExt};

impl AllocBuffers {
    /// Allocate buffers and fill them with the provided data
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &Device,
        projection: &Mat4x4,
        connections: &[u32],
        operations: &[u32],
        vertices: &[Vertex],
        indices: &[u32],
        instances: &[InstanceData],
        color_presets: &[ColorPreset],
        inits: &[u32],
    ) -> Self {
        let store = BufferUsages::STORAGE;
        let uni = BufferUsages::UNIFORM | BufferUsages::COPY_DST;
        let io = BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;
        let vtx = BufferUsages::VERTEX;
        let idx = BufferUsages::INDEX;

        Self {
            projection: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("projection"),
                contents: bytemuck::bytes_of(projection),
                usage: uni,
            }),
            connections: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("connections"),
                contents: bytemuck::cast_slice(connections),
                usage: store,
            }),
            operations: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("operations"),
                contents: bytemuck::cast_slice(operations),
                usage: store,
            }),
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices"),
                contents: bytemuck::cast_slice(vertices),
                usage: vtx,
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indices"),
                contents: bytemuck::cast_slice(indices),
                usage: idx,
            }),
            instances: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instances"),
                contents: bytemuck::cast_slice(instances),
                usage: store,
            }),
            color_presets: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("color_presets"),
                contents: bytemuck::cast_slice(color_presets),
                usage: store,
            }),
            ping_pong: [
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("ping"),
                    contents: bytemuck::cast_slice(inits),
                    usage: io,
                }),
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("pong"),
                    contents: bytemuck::cast_slice(inits),
                    usage: io,
                }),
            ],
        }
    }
}
