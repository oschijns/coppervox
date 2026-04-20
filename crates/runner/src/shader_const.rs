use bytemuck::NoUninit;

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

/*
/// Instance data
#[repr(C)]
#[derive(NoUninit, Default, Clone, Copy)]
pub struct InstanceData {
    /// Transform to apply to the mesh
    pub(crate) transform: Mat4x4,

    /// Index of the color set to pull from the colors buffer
    pub(crate) colors_index: u32,

    /// Offset where to read vertices indices from the index buffer
    pub(crate) mesh_offset: u32,

    /// Number of indices to read for a given mesh
    pub(crate) mesh_length: u32,

    /// Complementary padding
    pub(crate) _padding: u32,
}
*/

/// Colors for a component type
#[repr(C)]
#[derive(NoUninit, Default, Clone, Copy)]
pub struct ColorPreset {
    /// Color used when the component is fully off (0)
    pub(crate) off: Vec4,

    /// Color used when the component has low power (1)
    pub(crate) low: Vec4,

    /// Color used when the component has high power (u32::MAX)
    pub(crate) high: Vec4,
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
