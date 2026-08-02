/// Build mesh from quad data
pub mod mesher;

/// Build a single mesh from quad data
pub mod uni_mesher;

/// Process the model and extract data in the order expected by the GPU
pub mod processor;

/// Convert the data into types supported by the GPU
pub mod converter;

use crate::{
    loader::converter::{Processing, Rendering},
    shader_const::Op,
};
use coppervox_analyzer::Label;
use coppervox_common::{Model, Voxel};
use coppervox_config::{Config, component::CompType};
use kiss3d::{prelude::GpuMesh3d, procedural::RenderMesh};
use std::{cell::RefCell, rc::Rc};

/// Error reporting when accessing information
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("Analysis error: {0}")]
    Analysis(#[from] coppervox_analyzer::Error),

    #[error("No configuration defined for the voxel type {0}")]
    NoConfigVoxel(Voxel),

    #[error("No configuration defined for the component label {0}")]
    NoConfigLabel(Label),

    #[error("Node 0x{0:04x} in the incidence matrix is not mapped")]
    NodeIndexUnmapped(usize),

    #[error("Too many connections defined on a component")]
    TooManyConnections,
}

/// Reference-counted mesh
pub type RcMesh = Rc<RenderMesh>;

/// Reference to a GPU mesh, held in an `Rc` and protected by a `RefCell`
pub type RcGpuMesh = Rc<RefCell<GpuMesh3d>>;

/// Configuration loaded from the input files provided
pub struct Loader {
    /// Model loaded from the file
    model: Model,

    /// Configuration loaded from the text file
    config: Config,
}

/// Circuit split into processing data and rendering data
pub struct Circuit {
    /// Processing data
    process: Processing,

    /// Rendering data
    render: Rendering,
}

impl Loader {
    /// Create a new config
    #[inline]
    pub fn new(model: Model, config: Config) -> Self {
        Self { model, config }
    }
}

impl Circuit {
    /// Access the processing data for this circuit
    #[inline]
    pub fn processing(&self) -> &Processing {
        &self.process
    }

    /// Access the rendering data for this circuit
    #[inline]
    pub fn rendering(&self) -> &Rendering {
        &self.render
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
