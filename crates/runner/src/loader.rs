/// Build the runner
pub mod builder;

/// Build mesh from quad data
pub mod mesher;

use crate::shader_const::{ColorPreset, Op, Value};
use coppervox_common::Model;
use coppervox_config::{Config, component::CompType};
use kiss3d::scene::Object3d;

/// Configuration loaded from the input files provided
pub struct Loader {
    /// Model loaded from the file
    model: Model,

    /// Configuration loaded from the text file
    config: Config,
}

/// Circuit data loaded from the model and config
pub struct Circuit {
    /// Store connections between components
    connections: Vec<u32>,

    /// Store operations and the number of connections
    operations: Vec<u32>,

    /// Buffer of colors
    color_presets: Vec<ColorPreset>,

    /// Store the initial values for the inputs
    inits: Vec<Value>,

    // TODO: lookup instancing example
    /// Objects generated for the components to be rendered
    objects: Vec<Object3d>,
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
