/// Build the runner
pub mod builder;

/// Build mesh from quad data
pub mod mesher;

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

#[derive(Debug, Deserialize)]
pub struct ComponentConfig {
    /// Define the type of component
    #[serde(rename = "type")]
    comp_type: CompType,

    /// Colors of the component
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
    Input { input: Key, value: Value },
}

/// Define how a component should be colorized
#[derive(Debug, Default, Deserialize)]
pub struct ColorConfig {
    /// OFF color (0x0)
    off: Color,

    /// ON low power color (0x1)
    on_low: Color,

    /// ON high power color (0xFFFF)
    on_high: Color,
}

impl Loader {
    /// Create a new config
    #[inline]
    pub fn new(model: Model, config: Config) -> Self {
        Self { model, config }
    }
}
