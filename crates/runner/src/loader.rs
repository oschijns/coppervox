/// Build the runner
pub mod builder;

/// Build mesh from quad data
pub mod mesher;

use crate::shader_const::Op;
use coppervox_common::Model;
use coppervox_config::{Config, component::CompType};

/// Configuration loaded from the input files provided
pub struct Loader {
    /// Model loaded from the file
    model: Model,

    /// Configuration loaded from the text file
    config: Config,
}

impl Loader {
    /// Create a new config
    #[inline]
    pub fn new(model: Model, config: Config) -> Self {
        Self { model, config }
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
