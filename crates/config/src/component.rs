use crate::{VALUE_SIZE, Value, color::ColorSet};
use kiss3d::event::Key;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct CompDef {
    /// Define the type of component
    #[serde(rename = "type")]
    pub comp_type: CompType,

    /// Colors of the component
    pub color_set: ColorSet,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum CompType {
    /// constant value
    Const(Value) = 0,

    /// bitwise OR operation
    Or,

    /// bitwise AND operation
    And,

    /// bitwise NOR operation
    Nor,

    /// bitwise NAND operation
    Nand,

    /// arithmetic addition
    Add,

    /// arithmetic multiplication
    Mul,

    /// Minimum
    Min,

    /// Maximum
    Max,

    /// Map a key press to a single output value
    InputKey {
        /// The key to track
        key: Key,

        /// The value to output
        value: Value,
    },

    /// Map a list of keys to a mask
    InputMask([Option<Key>; VALUE_SIZE]),

    /// The component output a value
    Output(u8),
}

/// Define if a component serves as an interface to the outside world
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    /// The component does not interface with the outside world
    #[default]
    None = 0,

    /// The component serves as an input
    Input,

    /// The component serves as an output
    Output,
}

impl CompType {
    /// Get the start value for the component
    #[inline]
    pub fn start_value(&self) -> Value {
        match self {
            Self::Const(value) => *value,
            _ => 0,
        }
    }

    pub fn interface(&self) -> Interface {
        match self {
            Self::InputKey { key: _, value: _ } | Self::InputMask(_) => Interface::Input,
            Self::Output(_) => Interface::Output,
            _ => Interface::None,
        }
    }
}
