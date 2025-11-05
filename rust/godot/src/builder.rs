/// Build connectivity buffer from a incidence matrix
mod connect;

/// Read quads data to generate a mesh usable by Godot
mod gd_mesher;

use godot::prelude::*;

/// Operations defined in the compute shader
#[repr(C)]
#[derive(GodotConvert, Var, Export, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[godot(via = u32)]
pub enum Operation {
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
