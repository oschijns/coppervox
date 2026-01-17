use super::Loader;
use crate::loader::mesher::QuadDataReader;
use coppervox_analyzer::{
    Analysis, Mesher, Voxel,
    block_mesh::{QuadCoordinateConfig, RIGHT_HANDED_Y_UP_CONFIG},
};
use kiss3d::glamx::Vec3;
use petgraph::{
    Direction,
    graph::{DiGraph, NodeIndex},
    visit::IntoNodeIdentifiers,
};

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

/// Store data for running a logic circuit simulation
pub struct Runner {}

impl Loader {
    /// Build a logic circuit from the loaded model data
    pub fn build(&self) -> Result<Runner, Error> {
        // Config for generating meshes from quad data
        const QUAD_CONFIG: QuadCoordinateConfig = RIGHT_HANDED_Y_UP_CONFIG;
        let mut mesher = Mesher::new(QUAD_CONFIG, 0);
        let mut reader = QuadDataReader::new(true, false);

        // Analyse the model
        let analysis = Analysis::build(&self.model, &mut mesher, &mut reader);

        // How many component do we have in the simulation?
        let count = analysis.count();

        // Shapes that have been generated
        let shape_set = analysis.shape_set();

        // Iterate over each entry in the incidence matrix
        let incidences = analysis.connectivity().incidence_matrix();
        for node_index in incidences.node_identifiers() {
            // For the given node, get the associated voxel value
            let info = analysis.info_from_node_index(node_index)?;

            // Identify the type of the component
            let voxel = info.voxel_value;
            let Some(cfg) = self.config.0.get(&voxel) else {
                return Err(Error::NoConfig(voxel));
            };

            // position of the object
            let [x, y, z] = info.origin;
            let position = Vec3::new(x as f32, y as f32, z as f32);

            // Get the mesh to render for this component
            let mesh = shape_set.get_mesh(info.label)?;
        }

        // Add the edges connecting the nodes
        for node_index in incidences.node_identifiers() {
            for other_index in incidences.neighbors_directed(node_index, Direction::Outgoing) {}
        }

        Ok(Runner {})
    }
}
