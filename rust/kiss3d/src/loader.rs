/// Build the runner
pub mod builder;

/// Build mesh from quad data
pub mod mesher;

use crate::shader_const::Op;
use coppervox_analyzer::{Model, Voxel};
use kiss3d::{color::Color, event::Key};
use serde::Deserialize;
use std::collections::HashMap;

/// Configuration loaded from the input files provided
pub struct Loader {
    /// Model loaded from the file
    model: Model,

    /// Configuration loaded from the text file
    config: Config,
}

/// Configuration loaded from a text file such as TOML
#[derive(Debug, Deserialize)]
pub struct Config(HashMap<Voxel, ComponentConfig>);

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct ComponentConfig {
    /// Define the type of component
    #[serde(rename = "type")]
    comp_type: CompType,

    /// Colors of the component
    #[serde(flatten)]
    color: ColorConfig,
}

/// Value type processed in the components
pub type Value = u32;

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
        value: u32,
    },

    /// Map a list of keys to a mask
    InputMask([Option<Key>; 32]),

    /// The component output a value
    Output(u8),
}

/// Define how a component should be colorized
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
pub struct ColorConfig {
    /// OFF color (0x0)
    off: Color,

    /// ON low power color (0x1)
    on_low: Color,

    /// ON high power color (0xFFFF)
    on_high: Color,
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

impl Loader {
    /// Create a new config
    #[inline]
    pub fn new(model: Model, config: Config) -> Self {
        Self { model, config }
    }
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

    /// Get the operation value to submit to the GPU
    #[rustfmt::skip]
    pub fn to_op(&self) -> Op {
        match self {
            Self::Or   => Op::Or,
            Self::And  => Op::And,
            Self::Nor  => Op::Nor,
            Self::Nand => Op::Nand,
            Self::Add  => Op::Add,
            Self::Mul  => Op::Mul,
            Self::Min  => Op::Min,
            Self::Max  => Op::Max,
            _ => Op::None,
        }
    }

    /// Determine if the component serves as an interface
    pub fn interface(&self) -> Interface {
        match self {
            Self::InputKey { key: _, value: _ } | Self::InputMask(_) => Interface::Input,
            Self::Output(_) => Interface::Output,
            _ => Interface::None,
        }
    }
}
