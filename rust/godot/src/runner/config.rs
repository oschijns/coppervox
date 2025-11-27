use godot::prelude::*;

/// Maximum of connections allowed per component
pub const MAX_CONNECTIONS: usize = 32;

/// Storage buffer entry type size
pub const ELEMENT_SIZE: usize = size_of::<u32>();

/// Size of a workgroup in the compute shader
pub const WORKGROUP_SIZE: usize = 256;

/// Bit shift to apply to combine operation type and connections count
pub const OP_BIT_SHIFT: usize = 8;

/// Layouts for the compute shader
pub struct Layout {
    pub set: u32,
    pub binding: i32,
}

impl Layout {
    /// Create a new layout corresponding to the compute shader
    const fn new(set: u32, binding: i32) -> Self {
        Self { set, binding }
    }
}

/// Binding point for the number of components
pub const BIND_SIZE: Layout = Layout::new(0, 0);

/// Binding point for the incidence matrix
pub const BIND_CONNECTIONS: Layout = Layout::new(0, 1);

/// Binding point for the types of the components
pub const BIND_OPERATIONS: Layout = Layout::new(0, 2);

/// Binding point for the inputs
pub const BIND_INPUTS: Layout = Layout::new(1, 0);

/// Binding point for the outputs
pub const BIND_OUTPUTS: Layout = Layout::new(2, 0);

/// Binding point for the output sampler
pub const BIND_SAMPLER: Layout = Layout::new(2, 1);

/// Operations defined in the compute shader
#[repr(C)]
#[derive(GodotConvert, Var, Export, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[godot(via = i32)]
pub enum CVOperation {
    /// constant value
    Const = 0,

    /// bitwise OR operation
    Or = 1,

    /// bitwise AND operation
    And = 2,

    /// bitwise NOR operation
    Nor = 3,

    /// bitwise NAND operation
    Nand = 4,

    /// arithmetic addition
    Add = 5,

    /// arithmetic multiplication
    Mul = 6,

    /// Minimum
    Min = 7,

    /// Maximum
    Max = 8,
}
