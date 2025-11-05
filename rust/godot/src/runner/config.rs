use godot::prelude::*;

/// Size of a workgroup in the compute shader
pub const WORKGROUP_SIZE: u32 = 256;

/// Binding point for the number of components
pub const BIND_SIZE: i32 = 0;

/// Binding point for the incidence matrix
pub const BIND_CONNECTIONS: i32 = 1;

/// Binding point for the types of the components
pub const BIND_OPERATIONS: i32 = 2;

/// Binding point for the inputs
pub const BIND_INPUTS: i32 = 3;

/// Binding point for the outputs
pub const BIND_OUTPUTS: i32 = 4;

/// Binding point for the output sampler
pub const BIND_SAMPLER: i32 = 5;

/// Operations defined in the compute shader
#[repr(C)]
#[derive(GodotConvert, Var, Export, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[godot(via = i32)]
pub enum CVOperation {
    /// bitwise OR operation
    Or = 0,

    /// bitwise AND operation
    And = 1,

    /// bitwise NOR operation
    Nor = 2,

    /// bitwise NAND operation
    Nand = 3,

    /// arithmetic addition
    Add = 4,

    /// arithmetic multiplication
    Mul = 5,

    /// Minimum
    Min = 6,

    /// Maximum
    Max = 7,
}
