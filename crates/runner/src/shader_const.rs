use coppervox_common::rgb::{RGB8, Rgb};
use coppervox_config::color::ColorGradient;

/// Size of workgroups
pub const WORKGROUP: u32 = 256;

/// Maximum number of connections per output
pub const MAX_CNX: u64 = 16;

/// Size of the integer type used in the storage buffers
pub const INT_SIZE: u64 = 32 / 8;

/// Values transmitted over components are 32-bits
pub type GpuValue = u32;

/// Values transmitted over components are 32-bits
pub const VALUE_SIZE: usize = 32;

/// WGSL mat4x4<f32>
pub type GpuMat4x4 = [[f32; 4]; 4];

/// WGSL vec4<f32>
pub type GpuVec4 = [f32; 4];

/// WGSL vec3<f32>
pub type GpuVec3 = [f32; 3];

/// Instance data
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuInstanceData {
    /// Translation to apply to the mesh
    pub(crate) translate: GpuVec3,

    /// Index of the color set to pull from the colors buffer
    pub(crate) colors_index: u32,
}

impl GpuInstanceData {
    /// Create the per-instance data for one component
    pub fn new(translate: GpuVec3, colors_index: u32) -> Self {
        Self {
            translate,
            colors_index,
        }
    }
}

/// Colors for a component type
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuColorGradient {
    /// Color used when the component is fully off (0)
    pub(crate) off: GpuVec4,

    /// Color used when the component has low power (1)
    pub(crate) low: GpuVec4,

    /// Color used when the component has high power (u32::MAX)
    pub(crate) high: GpuVec4,
}

impl GpuColorGradient {
    /// Create a color preset for a component type
    pub fn new(off: GpuVec4, low: GpuVec4, high: GpuVec4) -> Self {
        Self { off, low, high }
    }
}

impl From<ColorGradient> for GpuColorGradient {
    fn from(color: ColorGradient) -> Self {
        Self::new(
            to_gpu_color(color.off.0),
            to_gpu_color(color.on_low.0),
            to_gpu_color(color.on_high.0),
        )
    }
}

/// Converts an RGBA color to a GPU-compatible color array.
#[inline]
fn to_gpu_color(color: RGB8) -> GpuVec4 {
    const F: f32 = 1.0 / u8::MAX as f32;

    let Rgb { r, g, b } = color;
    [r as f32 * F, g as f32 * F, b as f32 * F, 1.0]
}

#[repr(u32)]
#[derive(Debug, Default)]
pub enum GpuOp {
    /// Non operation
    #[default]
    None = 0,

    /// Bitwise OR operation
    Or = 1 << 24,

    /// Bitwise AND operation
    And = 2 << 24,

    /// Bitwise NOR operation
    Nor = 3 << 24,

    /// Bitwise NAND operation
    Nand = 4 << 24,

    /// Addition
    Add = 5 << 24,

    /// Multiplication
    Mul = 6 << 24,

    /// Minimum
    Min = 7 << 24,

    /// Maximum
    Max = 8 << 24,
}

/// Layouts for the compute shader
pub struct GpuLayout {
    /// Bind group index
    #[allow(unused)]
    pub group: u32,

    /// Binding index
    pub binding: u32,
}

impl GpuLayout {
    /// Create a new layout corresponding to the compute shader
    #[inline]
    pub const fn new(group: u32, binding: u32) -> Self {
        Self { group, binding }
    }
}
