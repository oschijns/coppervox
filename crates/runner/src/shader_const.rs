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

/// Define connections between outputs and inputs
pub const BIND_CONNECTIONS: Layout = Layout::new(0, 0);

/// Define the type of operation and the number of inputs to read for each output
pub const BIND_OPERATIONS: Layout = Layout::new(0, 1);

/// Input values to process
pub const BIND_INPUTS: Layout = Layout::new(1, 0);

/// Ouput values to write
pub const BIND_OUTPUTS: Layout = Layout::new(1, 1);

/// Layouts for the compute shader
pub struct Layout {
    pub set: u32,
    pub binding: u32,
}

impl Layout {
    /// Create a new layout corresponding to the compute shader
    #[inline]
    const fn new(set: u32, binding: u32) -> Self {
        Self { set, binding }
    }
}
