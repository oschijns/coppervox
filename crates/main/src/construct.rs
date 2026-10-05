use crate::{
    Cli,
    import::{import_model, load_config},
};
use coppervox_runner::{
    config::identify::VoxelIdentifier,
    loader::{
        converter::{Processing, RenderingData},
        processor::ProcessingData,
    },
    runner::{Runner, RunnerData},
    shader_const::{GpuColorGradient, GpuMat4x4},
};
use kiss3d::{glamx::Vec3, procedural::RenderMesh, resource::Material3d};
use std::{cell::RefCell, rc::Rc};

/// Identify matrix to prepare the projection matrix for the camera
const IDENTITY_MATRIX: GpuMat4x4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Circuit data to construct
pub struct Construct {
    /// Mesh depicting the circuit
    pub render_mesh: RenderMesh,

    /// Material based on a Runner
    pub material: Rc<RefCell<Box<dyn Material3d + 'static>>>,

    /// Center of the model
    pub center: Vec3,
}

impl Construct {
    /// Process the data to construct the circuit
    pub fn new(cli: &Cli) -> Result<Self, anyhow::Error> {
        // Load the configuration provided
        let config = load_config(&cli.config)?;
        let mut vox_ident = VoxelIdentifier::make(config);

        // Load the voxel data
        const CHUNK_DIM: [usize; 3] = [12, 12, 12];
        let model = import_model(&cli.model, &mut vox_ident, CHUNK_DIM)?;

        // Process the data to build the logic circuit and the rendered mesh
        let ordered = ProcessingData::make(&model, &vox_ident)?;
        let process = Processing::make(ordered.io_order(), ordered.connect());

        let color_presets = vox_ident
            .definitions
            .iter()
            .map(|(_, g)| GpuColorGradient::from(*g))
            .collect::<Box<[_]>>();
        let render = RenderingData::make(ordered.quad_data(), ordered.meshes(), color_presets);

        // Get the center of the circuit
        let [x, y, z] = ordered.center();
        let center = Vec3::new(x as f32, y as f32, z as f32) * 0.5;

        // Group runner data
        let data = RunnerData::new(
            IDENTITY_MATRIX,
            process.connections.into_boxed_slice(),
            process.operations.into_boxed_slice(),
            render.instances_data,
            render.color_presets,
            process.initial_values.into_boxed_slice(),
        );

        Ok(Self {
            render_mesh: render.render_mesh,
            material: Rc::new(RefCell::new(
                Box::new(Runner::new(data)) as Box<dyn Material3d + 'static>
            )),
            center,
        })
    }
}
