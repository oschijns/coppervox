use super::{Analysis, Connectivity, Error, Info, Label, LabeledModel, QuadsReader};
use crate::{Model, analyzer::meshing::Mesher};
use gridmap::hashbrown::HashMap;
use petgraph::matrix_graph::DiMatrix;

impl<M> Analysis<M> {
    /// Analyze the voxel model
    pub fn build<R>(model: &Model, mesher: &mut Mesher, reader: &mut R) -> Self
    where
        R: QuadsReader<M>,
    {
        // labelize the model
        let mut labeled_model = LabeledModel::from_model(model);

        // identify the shapes of the components and convert them into meshes
        let shapes = labeled_model.identify_shapes();
        let mut meshes = HashMap::with_capacity(shapes.len());
        for (&hash, shape) in shapes.iter() {
            meshes.insert(hash, mesher.build_mesh(shape, reader));
        }

        // identify connections between islands
        let connectivity = Connectivity::from_model(&labeled_model);

        Self {
            labeled_model,
            meshes,
            connectivity,
        }
    }
}

impl<M> Analysis<M>
where
    M: Clone,
{
    /// Get information on the component
    pub fn get_info(&self, label: Label) -> Result<Info<M>, Error> {
        // Check if an island was defined for this label
        let Some(island) = self.labeled_model.islands.get(&label) else {
            return Err(Error::NoInfo);
        };

        // Check if a mesh was generated for this label
        let Some(mesh) = island
            .shape_hash
            .and_then(|hash| self.meshes.get(&hash).cloned())
        else {
            return Err(Error::NoMesh);
        };

        // Check if a graph node was assigned to this label
        let Some(&node_index) = self.connectivity.mapping.get(&label) else {
            return Err(Error::NotConnected);
        };

        Ok(Info {
            voxel_value: island.voxel_value,
            voxels_count: island.voxels_count,
            mesh,
            node_index,
        })
    }

    /// Get connectivity of the components
    #[inline]
    pub fn get_connectivity_matrix(&self) -> &DiMatrix<Label, usize> {
        &self.connectivity.graph
    }
}
