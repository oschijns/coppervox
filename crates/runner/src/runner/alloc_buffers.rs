/*
 * Since modern CPU and GPU both works in little endian,
 * we do not need to swap endianness when submitting buffers to the CPU.
 * Either way Vulkan require CPU and GPU to share the same endianness.
 */

use super::*;
use bytemuck::NoUninit;
use coppervox_config::color::ColorSet;
use kiss3d::prelude::*;
use wgpu::BufferUsages;

/// Component data
#[repr(C)]
#[derive(NoUninit, Clone, Copy)]
pub struct Matrix(pub(crate) [[f32; 4]; 4]);

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
        projection: &Matrix,
        connections: &[u32],
        operations: &[u32],
        transforms: &[ComponentData],
        colors: &[Colors],
        inits: &[u32],
    ) -> Self {
        let s = BufferUsages::STORAGE;
        let i = BufferUsages::STORAGE | BufferUsages::COPY_SRC;
        let io = BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;

        Self {
            projection: ctx.create_buffer_init(
                Some("projection"),
                bytemuck::bytes_of(projection),
                i,
            ),
            connections: ctx.create_buffer_init(
                Some("connections"),
                bytemuck::cast_slice(connections),
                s,
            ),
            operations: ctx.create_buffer_init(
                Some("operations"),
                bytemuck::cast_slice(operations),
                s,
            ),
            transforms: ctx.create_buffer_init(
                Some("transforms"),
                bytemuck::cast_slice(transforms),
                s,
            ),
            colors: ctx.create_buffer_init(Some("colors"), bytemuck::cast_slice(colors), s),
            ping_pong: [
                ctx.create_buffer_init(Some("ping"), bytemuck::cast_slice(inits), io),
                ctx.create_buffer_init(Some("pong"), bytemuck::cast_slice(inits), io),
            ],
        }
    }
}

/// Convert a color set into a data struct to be passed to the shader
impl From<ColorSet> for Colors {
    fn from(set: ColorSet) -> Self {
        Self {
            off: convert_color(set.off),
            low: convert_color(set.on_low),
            high: convert_color(set.on_high),
        }
    }
}

/// Convert a RGBA byte into a RGBA float
#[inline]
fn convert_color(color: Rgba<u8>) -> [f32; 4] {
    // remap values in the range [0, 255] to values in the range [0.0, 1.0]
    const F: f32 = 1.0 / u8::MAX as f32;
    macro_rules! c {
        ($field:ident) => {
            (color.$field as f32) * F
        };
    }
    [c!(r), c!(g), c!(b), c!(a)]
}
