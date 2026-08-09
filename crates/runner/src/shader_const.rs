use coppervox_config::color::ColorSet;
use kiss3d::color::Rgba;

/// Size of workgroups
pub const WORKGROUP: u32 = 256;

/// Maximum number of connections per output
pub const MAX_CNX: u64 = 16;

/// Size of the integer type used in the storage buffers
pub const INT_SIZE: u64 = 32 / 8;

/// Values transmitted over components are 32-bits
pub type Value = u32;

/// Values transmitted over components are 32-bits
pub const VALUE_SIZE: usize = 32;

/// WGSL mat4x4<f32>
pub type Mat4x4 = [[f32; 4]; 4];

/// WGSL vec4<f32>
pub type Vec4 = [f32; 4];

/// WGSL vec3<f32>
pub type Vec3 = [f32; 3];

/// Instance data
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    /// Translation to apply to the mesh
    pub(crate) translate: Vec3,

    /// Index of the color set to pull from the colors buffer
    pub(crate) colors_index: u32,
}

impl InstanceData {
    /// Create the per-instance data for one component
    pub fn new(translate: Vec3, colors_index: u32) -> Self {
        Self {
            translate,
            colors_index,
        }
    }
}

/// Colors for a component type
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorPreset {
    /// Color used when the component is fully off (0)
    pub(crate) off: Vec4,

    /// Color used when the component has low power (1)
    pub(crate) low: Vec4,

    /// Color used when the component has high power (u32::MAX)
    pub(crate) high: Vec4,
}

impl ColorPreset {
    /// Create a color preset for a component type
    pub fn new(off: Vec4, low: Vec4, high: Vec4) -> Self {
        Self { off, low, high }
    }
}

impl From<ColorSet> for ColorPreset {
    fn from(color: ColorSet) -> Self {
        Self::new(
            to_gpu_color(color.off),
            to_gpu_color(color.on_low),
            to_gpu_color(color.on_high),
        )
    }
}

/// Converts an RGBA color to a GPU-compatible color array.
#[inline]
fn to_gpu_color(color: Rgba<u8>) -> [f32; 4] {
    let c = Rgba::<f32>::from(color);
    [c.r, c.g, c.b, c.a]
}

#[repr(u32)]
#[derive(Debug, Default)]
pub enum Op {
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
pub struct Layout {
    /// Bind group index
    #[allow(unused)]
    pub group: u32,

    /// Binding index
    pub binding: u32,
}

impl Layout {
    /// Create a new layout corresponding to the compute shader
    #[inline]
    pub const fn new(group: u32, binding: u32) -> Self {
        Self { group, binding }
    }
}
