use kiss3d::prelude::*;
use std::mem;
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ShaderStages,
};

use crate::shader_const::{BIND_CONNECTIONS, INT_SIZE, MAX_CNX};

/// Store data for running a logic circuit simulation
pub struct Runner {
    /// Binding for the connections storage buffer
    connections: GpuData,

    /// Binding for the operations storage buffer
    operations: GpuData,
    // Binding for the inputs storage buffer
    //inputs: GpuData,

    // Binding for the outputs storage buffer
    //outputs: GpuData,
}

/// Define the GPU data
pub struct GpuData {
    /// Buffer of data to send to the GPU
    buffer: Buffer,

    /// Binding layout
    layout: BindGroupLayout,

    /// Bind group for the storage buffer
    bind_group: Option<BindGroup>,
}

impl Runner {
    pub fn new(ctx: &Context, connections: &[u32], operations: &[u32]) -> Self {
        Self {
            connections: GpuData::read_only(ctx, "connections", connections),
            operations: GpuData::read_only(ctx, "operations", operations),
        }
    }
}

impl GpuData {
    /// Create buffers
    fn read_only(ctx: &Context, label: &str, data: &[u32]) -> Self {
        // Allocate a buffer
        let buffer = ctx.create_buffer(&BufferDescriptor {
            label: Some(label),
            size: data.len() as u64 * INT_SIZE,
            usage: BufferUsages::STORAGE, // | BufferUsages::COPY_SRC | BufferUsages::MAP_READ
            mapped_at_creation: false,
        });

        // Write the data to the buffer.
        // Modern CPU and GPU both works in little endian,
        // so no need to swap endianness here.
        ctx.write_buffer(&buffer, 0, bytemuck::cast_slice(data));

        // Create a layout for the buffer
        let layout = ctx.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(label),
            entries: &[BindGroupLayoutEntry {
                binding: BIND_CONNECTIONS.binding,
                visibility: ShaderStages::COMPUTE, // FRAGMENT
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Return descriptor for the buffer
        Self {
            buffer,
            layout,
            bind_group: None,
        }
    }
}
