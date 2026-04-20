/// Build the runner
pub mod builder;

/// Build mesh from quad data
pub mod mesher;

use std::{cell::RefCell, rc::Rc};

use crate::shader_const::{ColorPreset, Op, Value};
use coppervox_analyzer::ShapeHash;
use coppervox_common::{Model, Voxel};
use coppervox_config::{Config, component::CompType};
use kiss3d::{prelude::GpuMesh3d, scene::Object3d};

/// Error reporting when accessing information
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("{0}")]
    Analysis(#[from] coppervox_analyzer::Error),

    #[error("There is an inconsistency in the mesh associated with a component {0:#08x}")]
    MeshInconsistency(ShapeHash),

    #[error("No configuration defined for the voxel type {0}")]
    NoConfig(Voxel),

    #[error("Node 0x{0:04x} in the incidence matrix is not mapped")]
    NodeIndexUnmapped(usize),

    #[error("Too many connections defined on a component")]
    TooManyConnections,
}

/// Configuration loaded from the input files provided
pub struct Loader {
    /// Model loaded from the file
    model: Model,

    /// Configuration loaded from the text file
    config: Config,
}

/// Circuit data to be used by the process shader
pub struct CircuitData {
    /// Store connections between components
    connections: Vec<u32>,

    /// Store operations and the number of connections
    operations: Vec<(Op, u32)>,

    /// Store the initial values for the inputs
    inits: Vec<Value>,

    /// Range of the components in the circuits
    io_ranges: IoRanges,
}

/// Circuit data to be used by the render shader
pub struct CircuitRender {
    /// Buffer of colors
    color_presets: Vec<ColorPreset>,

    /// Objects generated for the components to be rendered
    objects: Vec<Object3d>,
}

/// Reference to a GPU mesh, held in an `Rc` and protected by a `RefCell`
pub type RefGpuMesh3d = Rc<RefCell<GpuMesh3d>>;

/// Define ranges for inputs and outputs components in the circuit
#[derive(Debug, Clone, Copy)]
pub struct IoRanges {
    /// Range of the components used as inputs
    pub inputs: [usize; 2],

    /// Range of the components used as outputs
    pub outputs: [usize; 2],
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
