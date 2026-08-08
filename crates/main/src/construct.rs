use crate::{Cli, import::import_model};
use coppervox_runner::{
    config::Config,
    loader::{
        converter::{Processing, to_gpu_color},
        processor::OrderedData,
        uni_mesher::UniQuadDataReader,
    },
    runner::{Runner, RunnerData},
    shader_const::{ColorPreset, InstanceData, Mat4x4},
};
use kiss3d::{
    procedural::RenderMesh,
    resource::{Material3d, Texture},
};
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
    pub mesh: RenderMesh,

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

        // Convert color presets
        let color_presets = config
            .colors
            .iter()
            .map(|c| ColorPreset::from(*c))
            .collect::<Vec<_>>();

        // Helper for building the mesh data
        let mut mesher = UniQuadDataReader::new();

        // Instance data with translation and color index
        let mut instances_data = Vec::with_capacity(process.operations().len());
        for (_, (_, _, color_index, bbox)) in ordered.meshes().components().iter().enumerate() {
            let [x, y, z] = bbox.start;
            instances_data.push(InstanceData::new(
                [x as f32, y as f32, z as f32],
                *color_index as u32,
            ));
        }

        // Group runner data
        let data = RunnerData::new(
            IDENTITY_MATRIX,
            process.connections,
            process.operations,
            instances_data,
            color_presets,
            process.initial_values,
        );

        let material = Rc::new(RefCell::new(
            Box::new(Runner::new(data)) as Box<dyn Material3d + 'static>
        ));

        todo!()
    }
}
