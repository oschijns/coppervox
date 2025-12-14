/// Hardcoded shader config
pub mod shader;

/// Identify inputs
pub mod input;

use serde::Deserialize;

/// Value type processed in the components
pub type Value = u32;

/// Initial value
pub const INIT: Value = 0;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum CompType {
    /// constant value
    Const(Value) = 0,

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

    /// Input value
    Input { input: input::Input, value: Value },
}

/// Default to a component that returns zero
impl Default for CompType {
    #[inline]
    fn default() -> Self {
        Self::Const(INIT)
    }
}
