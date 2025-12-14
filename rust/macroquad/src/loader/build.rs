use super::Config;
use crate::{
    loader::mesher::QuadDataReader,
    runner::{Component, Runner},
};
use coppervox_analyzer::{
    Analysis, Error, Mesher,
    block_mesh::{QuadCoordinateConfig, RIGHT_HANDED_Y_UP_CONFIG},
};
use macroquad::math::Vec3;
use petgraph::visit::IntoNodeIdentifiers;

impl Config {
    /// Build a logic circuit from the loaded model data
    pub fn build(&self) -> Result<Runner, Error> {
        // Config for generating meshes from quad data
        const QUAD_CONFIG: QuadCoordinateConfig = RIGHT_HANDED_Y_UP_CONFIG;
        let mut mesher = Mesher::new(QUAD_CONFIG, 0);
        let mut reader = QuadDataReader::new(false);

        // Analyse the model
        let analysis = Analysis::build(&self.model, &mut mesher, &mut reader);

        // How many component do we have in the simulation?
        let count = analysis.count();
        let mut components = Vec::with_capacity(count);

        // Shapes that have been generated
        let shape_set = analysis.shape_set();

        // Iterate over each entry in the incidence matrix
        let incidences = analysis.connectivity().incidence_matrix();
        for node_index in incidences.node_identifiers() {
            // For the given node, get the associated voxel value
            if let Ok(info) = analysis.info_from_node_index(node_index) {
                // Identify the type of the component
                let comp_type = self.map.get(&info.voxel_value).cloned().unwrap_or_default();

                // position of the object
                let [x, y, z] = info.origin;
                let position = Vec3::new(x as f32, y as f32, z as f32);

                // Get the mesh to render for this component
                let mesh = shape_set.get_mesh(info.label)?;

                // Add the new component to the list
                components.push(Component::new(comp_type, position, mesh.clone()));
            }
        }

        todo!()
    }
}
