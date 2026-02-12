use super::Loader;
use crate::loader::{ComponentConfig, Interface, mesher::QuadDataReader};
use coppervox_analyzer::{
    Analysis, IslandInfo, Label, Mesher, Voxel,
    block_mesh::{QuadCoordinateConfig, RIGHT_HANDED_Y_UP_CONFIG},
};
use kiss3d::procedural::RenderMesh;
use std::sync::Arc;

/// Error reporting when accessing information
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("{0}")]
    Analysis(#[from] coppervox_analyzer::Error),

    #[error("No configuration defined for the voxel type {0}")]
    NoConfig(Voxel),

    #[error("Node index in the graphes do not match ({initial} != {new})")]
    NodeIndexMismatch { initial: usize, new: usize },
}

/// Define an ordering for the components
struct Ordering {
    /// Define the order of the components
    order: Vec<Component>,

    /// Range of the components used as inputs
    range_inputs: [usize; 2],

    /// Range of the components used as outputs
    range_outputs: [usize; 2],
}

/// Group all the relevant data of a component
#[derive(Clone)]
struct Component {
    /// Identifier of the component
    label: Label,

    /// Corresponding config
    config: ComponentConfig,

    /// Voxel information
    info: IslandInfo,

    /// The mesh to render
    mesh: Arc<RenderMesh>,
}

impl Loader {
    /// Build a logic circuit from the loaded model data
    pub fn build(&self) -> Result<(), Error> {
        // Config for generating meshes from quad data
        const QUAD_CONFIG: QuadCoordinateConfig = RIGHT_HANDED_Y_UP_CONFIG;
        let mut mesher = Mesher::new(QUAD_CONFIG, 0);
        let mut reader = QuadDataReader::new(true, false);

        // Analyse the model
        let analysis = Analysis::build(&self.model, &mut mesher, &mut reader);
        let order = self.get_order(&analysis)?;

        Ok(())
    }

    /// Define an ordering for the components.
    /// This will allow to optimize buffer copies between the CPU and the GPU.
    fn get_order(&self, analysis: &Analysis<Arc<RenderMesh>>) -> Result<Ordering, Error> {
        // How many component do we have in the simulation?
        let count = analysis.count();

        // We want to organize components into three groups
        let mut group_nones = Vec::with_capacity(count);
        let mut group_inputs = Vec::with_capacity(count);
        let mut group_outputs = Vec::with_capacity(count);

        // Iterate over the components
        for (&label, &info) in analysis.labeled_model().iter() {
            // Identify the type of the component
            let voxel = info.voxel_value();
            let Some(&config) = self.config.0.get(&voxel) else {
                return Err(Error::NoConfig(voxel));
            };

            // Get the mesh to render for this component
            let mesh = analysis.shape_set().get_mesh(label)?;

            // Deduce to which group the component should be added
            let group = match config.comp_type.interface() {
                Interface::None => &mut group_nones,
                Interface::Input => &mut group_inputs,
                Interface::Output => &mut group_outputs,
            };

            // Get most data related to the component
            group.push(Component {
                label,
                config,
                info,
                mesh: mesh.clone(),
            });
        }

        // We are putting the groups in the following order:
        // - Inputs
        // - Outputs
        // - Internals

        // Compute the ranges for the interfaces
        let count_in = group_inputs.len();
        let count_out = group_outputs.len();
        let range_inputs = [0usize, count_in];
        let range_outputs = [count_in, count_in + count_out];

        // Compute the final list
        let mut order = group_inputs;
        order.append(&mut group_outputs);
        order.append(&mut group_nones);

        // Return the result
        Ok(Ordering {
            order,
            range_inputs,
            range_outputs,
        })
    }
}
