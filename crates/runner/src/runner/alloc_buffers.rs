/*
 * Since modern CPU and GPU both works in little endian,
 * we do not need to swap endianness when submitting buffers to the CPU.
 * Either way Vulkan require CPU and GPU to share the same endianness.
 */

use super::*;
use bytemuck::NoUninit;
use kiss3d::prelude::*;
use wgpu::BufferUsages;

/// Component data
#[repr(C)]
#[derive(NoUninit, Clone, Copy)]
pub struct ComponentData {
    pub(crate) transform: [[f32; 4]; 4],
    pub(crate) type_index: u32,
    pub(crate) _padding: [u32; 3],
}

/// Colors for a component type
#[repr(C)]
#[derive(NoUninit, Clone, Copy)]
pub struct Colors {
    pub(crate) off: [f32; 4],
    pub(crate) low: [f32; 4],
    pub(crate) high: [f32; 4],
}

impl AllocBuffers {
    /// Allocate buffers and fill them with the provided data
    pub fn new(
        ctx: &Context,
        connections: &[u32],
        operations: &[u32],
        components: &[ComponentData],
        colors: &[Colors],
        inits: &[u32],
    ) -> Self {
        let ro = BufferUsages::STORAGE;
        let rw = BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;

        Self {
            connections: ctx.create_buffer_init(
                Some("connections"),
                bytemuck::cast_slice(connections),
                ro,
            ),
            operations: ctx.create_buffer_init(
                Some("operations"),
                bytemuck::cast_slice(operations),
                ro,
            ),
            components: ctx.create_buffer_init(
                Some("components"),
                bytemuck::cast_slice(components),
                ro,
            ),
            colors: ctx.create_buffer_init(Some("colors"), bytemuck::cast_slice(colors), ro),
            ping_pong: [
                ctx.create_buffer_init(Some("ping"), bytemuck::cast_slice(inits), rw),
                ctx.create_buffer_init(Some("pong"), bytemuck::cast_slice(inits), rw),
            ],
        }
    }
}
