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

    /// Camera position relative to the centered model
    pub camera_offset: Vec3,
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
        if render.instances_data.is_empty() {
            anyhow::bail!("The imported model contains no renderable components");
        }

        // Center on the actual bounds, which may not start at the origin.
        let bounds = ordered.bounding_box();
        let [min_x, min_y, min_z] = bounds.start;
        let [max_x, max_y, max_z] = bounds.end;
        let center = Vec3::new(
            ((min_x as f64 + max_x as f64) * 0.5) as f32,
            ((min_y as f64 + max_y as f64) * 0.5) as f32,
            ((min_z as f64 + max_z as f64) * 0.5) as f32,
        );

        let size_x = (max_x as f64 - min_x as f64).max(1.0);
        let size_y = (max_y as f64 - min_y as f64).max(1.0);
        let size_z = (max_z as f64 - min_z as f64).max(1.0);
        let camera_distance =
            ((size_x * size_x + size_y * size_y + size_z * size_z).sqrt() * 1.75).max(4.0) as f32;
        let camera_offset = Vec3::splat(camera_distance / 3.0_f32.sqrt());

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
            camera_offset,
        })
    }
}
