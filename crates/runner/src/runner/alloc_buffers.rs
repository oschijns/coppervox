/*
 * Since modern CPU and GPU both works in little endian,
 * we do not need to swap endianness when submitting buffers to the CPU.
 * Either way Vulkan require CPU and GPU to share the same endianness.
 */

use super::*;
use crate::shader_const::{ColorPreset, InstanceData, Mat4x4};
use coppervox_config::color::ColorSet;
use kiss3d::prelude::*;
use wgpu::BufferUsages;

impl AllocBuffers {
    /// Allocate buffers and fill them with the provided data
    pub fn new(
        ctx: &Context,
        projection: &Mat4x4,
        connections: &[u32],
        operations: &[u32],
        instances_data: &[InstanceData],
        color_presets: &[ColorPreset],
        inits: &[u32],
    ) -> Self {
        let store = BufferUsages::STORAGE;
        let uni = BufferUsages::UNIFORM | BufferUsages::COPY_DST;
        let io = BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;

        Self {
            projection: ctx.create_buffer_init(
                Some("projection"),
                bytemuck::bytes_of(projection),
                uni,
            ),
            connections: ctx.create_buffer_init(
                Some("connections"),
                bytemuck::cast_slice(connections),
                store,
            ),
            operations: ctx.create_buffer_init(
                Some("operations"),
                bytemuck::cast_slice(operations),
                store,
            ),
            instances_data: ctx.create_buffer_init(
                Some("instances_data"),
                bytemuck::cast_slice(instances_data),
                store,
            ),
            color_presets: ctx.create_buffer_init(
                Some("color_presets"),
                bytemuck::cast_slice(color_presets),
                store,
            ),
            ping_pong: [
                ctx.create_buffer_init(Some("ping"), bytemuck::cast_slice(inits), io),
                ctx.create_buffer_init(Some("pong"), bytemuck::cast_slice(inits), io),
            ],
        }
    }
}

/// Convert a color set into a data struct to be passed to the shader
impl From<ColorSet> for ColorPreset {
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
