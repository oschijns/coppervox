use super::Builder;
use coppervox_analyzer::{Analysis, ShapeSet};
use godot::{
    classes::{ArrayMesh, RenderingServer},
    prelude::*,
};
use petgraph::visit::IntoNodeIdentifiers;

impl Builder {
    /// Assign the material to the meshes generated
    pub fn assign_material_to_meshes(&self, shape_set: &ShapeSet<Gd<ArrayMesh>>) -> Result<(), ()> {
        // If no material was set, this function cannot work
        if let Some(material) = &self.material {
            for mesh in shape_set.get_meshes() {
                mesh.clone().surface_set_material(0, material);
            }
            Ok(())
        } else {
            Err(())
        }
    }

    /// Instantiate the meshes to be rendered
    pub fn instantiate_meshes(&mut self, analysis: &Analysis<Gd<ArrayMesh>>) {
        let mut render = RenderingServer::singleton();

        // iterate over each node
        for node_index in analysis
            .connectivity()
            .incidence_matrix()
            .node_identifiers()
        {
            // For the given node, get the associated voxel value
            if let Ok(info) = analysis.info_from_node_index(node_index) {
                let [x, y, z] = info.origin;
                let origin = Vector3::new(x as f32, y as f32, z as f32);

                // Create a mesh instance, position it in space
                // and set its  index in the rendering shader.
                let inst = render.instance_create();
                render.instance_set_transform(inst, Transform3D::new(Basis::IDENTITY, origin));
                render.instance_set_base(inst, info.mesh.get_rid());
                render.instance_geometry_set_shader_parameter(
                    inst,
                    self.instance_index_shader_parameter.arg(),
                    &Variant::from(node_index.index() as u64),
                );
            }
        }
    }
}
