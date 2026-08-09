use crate::{Cli, import::import_model};
use coppervox_runner::{
    config::Config,
    loader::{
        converter::{Processing, Rendering},
        processor::OrderedData,
    },
    runner::{Runner, RunnerData},
    shader_const::Mat4x4,
};
use kiss3d::resource::{GpuMesh3d, Material3d};
use std::{cell::RefCell, fs, rc::Rc};

/// Identify matrix to prepare the projection matrix for the camera
const IDENTITY_MATRIX: Mat4x4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Circuit data to construct
pub struct Construct {
    /// Mesh depicting the circuit
    pub gpu_mesh: Rc<RefCell<GpuMesh3d>>,

    /// Material based on a Runner
    pub material: Rc<RefCell<Box<dyn Material3d + 'static>>>,
}

impl Construct {
    /// Process the data to construct the circuit
    pub fn new(cli: &Cli) -> Result<Self, anyhow::Error> {
        // Load the voxel data
        const CHUNK_DIM: [usize; 3] = [12, 12, 12];
        let model = import_model(&cli.model, CHUNK_DIM)?;

        // Load the configuration provided
        let content = fs::read_to_string(&cli.config)?;
        let config: Config = toml::from_str(&content)?;

        // Process the data to build the logic circuit and the rendered mesh
        let ordered = OrderedData::make(&model, &config)?;
        let process = Processing::make(ordered.io_order(), ordered.connect());
        let render = Rendering::make(ordered.quad_data(), ordered.meshes(), &config.colors);

        let gpu_mesh = render.gpu_mesh().clone();

        // Group runner data
        let data = RunnerData::new(
            IDENTITY_MATRIX,
            process.connections,
            process.operations,
            render.instances_data,
            render.color_presets,
            process.initial_values,
        );

        let material = Rc::new(RefCell::new(
            Box::new(Runner::new(data)) as Box<dyn Material3d + 'static>
        ));

        Ok(Self { gpu_mesh, material })
    }
}
