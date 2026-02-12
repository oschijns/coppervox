use super::{Builder, BuilderError, ComponentType, RenderConfig, VoxelQualifier};
use crate::{builder::buffer::GeneratedBuffers, runner::config::CVOperation};
use coppervox_analyzer::ShapeSet;
use godot::{
    classes::{ArrayMesh, RenderingServer},
    prelude::*,
};
use petgraph::visit::IntoNodeIdentifiers;

impl Builder {
    /// Instantiate the meshes to be rendered and build the GPU buffers
    pub fn generate_internals(&mut self, contact_threshold: usize) -> Result<(), BuilderError> {
        let mut render = RenderingServer::singleton();

        // check that all the necessary resources have been set
        let Some(render_config) = self.render_config.as_ref().map(|c| c.bind()) else {
            return Err(BuilderError::MissingRenderConfig);
        };
        let Some(qualifier) = self.qualifier.as_ref().map(|c| c.bind()) else {
            return Err(BuilderError::MissingQualifier);
        };
        let Some(converter) = self.converter.as_ref().map(|c| c.bind()) else {
            return Err(BuilderError::MissingConverter);
        };
        let Some(analysis) = converter.analysis() else {
            return Err(BuilderError::NoAnalysis);
        };

        // Generate the raw buffers
        self.buffers = Some(GeneratedBuffers::build(
            analysis,
            &qualifier,
            contact_threshold,
        )?);

        // Assign the material to the meshes
        render_config.assign_material_to_meshes(analysis.shape_set())?;

        // allocate space for the instantiated meshes
        let incidence = analysis.connectivity().incidence_matrix();
        self.instantiated_meshes.reserve(incidence.node_count());

        // iterate over each node
        for node_index in incidence.node_identifiers() {
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
                    render_config.instance_index_shader_parameter.arg(),
                    &Variant::from(node_index.index() as u64),
                );

                // Store the instance for later deletion
                self.instantiated_meshes.push(inst);
            }
        }

        Ok(())
    }

    /// Delete all instantiated meshes
    pub fn delete_meshes(&mut self) {
        let mut render = RenderingServer::singleton();
        for &inst in self.instantiated_meshes.iter() {
            render.free_rid(inst);
        }
        self.instantiated_meshes.clear();
    }
}

impl RenderConfig {
    /// Assign the material to the meshes generated
    pub fn assign_material_to_meshes(
        &self,
        shape_set: &ShapeSet<Gd<ArrayMesh>>,
    ) -> Result<(), BuilderError> {
        // If no material was set, this function cannot work
        if let Some(material) = &self.material {
            for mesh in shape_set.get_meshes() {
                mesh.clone().surface_set_material(0, material);
            }
            Ok(())
        } else {
            Err(BuilderError::MissingMaterial)
        }
    }
}

impl VoxelQualifier {
    /// When a new voxel mapping is applied to this resource,
    /// recompute the internal model.
    pub(crate) fn update_mapping(&mut self) {
        for map in self.mapping.iter_shared() {
            let map = map.bind();
            let key = map.voxel_type;

            if map.action.is_empty() {
                if map.operation == CVOperation::Const {
                    // The component is a constant value
                    self.evaluated
                        .insert(key, ComponentType::Constant(map.voxel_value));
                } else {
                    // The component represent an operation
                    self.evaluated
                        .insert(key, ComponentType::Operation(map.operation));
                }
            } else {
                // The component is activated by an action
                let action = map.action.clone();
                self.actions.insert(action.clone());
                self.evaluated.insert(
                    key,
                    ComponentType::Action {
                        value: map.voxel_value,
                        action,
                    },
                );
            }
        }
    }
}
