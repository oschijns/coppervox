/// Build the runner
pub mod builder;

/// Build mesh from quad data
pub mod mesher;

use crate::shader_const::{Op, Value};
use coppervox_analyzer::Model;
use coppervox_importer::config::{CompType, Config};

/// Configuration loaded from the input files provided
pub struct Loader {
    /// Model loaded from the file
    model: Model,

    /// Configuration loaded from the text file
    config: Config,
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

/// Get the start value for the component
#[inline]
pub fn start_value(comp_type: &CompType) -> Value {
    match comp_type {
        CompType::Const(value) => *value,
        _ => 0,
    }
}

impl Op {
    /// Get the operation value to submit to the GPU
    #[rustfmt::skip]
    pub fn from_type(comp_type: &CompType) -> Self {
        match comp_type {
            CompType::Or   => Self::Or,
            CompType::And  => Self::And,
            CompType::Nor  => Self::Nor,
            CompType::Nand => Self::Nand,
            CompType::Add  => Self::Add,
            CompType::Mul  => Self::Mul,
            CompType::Min  => Self::Min,
            CompType::Max  => Self::Max,
            _ => Self::None,
        }
    }
}

impl Interface {
    /// Determine if the component serves as an interface
    pub fn from_type(comp_type: &CompType) -> Self {
        match comp_type {
            CompType::InputKey { key: _, value: _ } | CompType::InputMask(_) => Self::Input,
            CompType::Output(_) => Self::Output,
            _ => Self::None,
        }
    }
}
