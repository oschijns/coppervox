use super::{Analysis, Connectivity, Error, Info, Label, LabeledModel, QuadsReader};
use crate::{
    Model,
    analyzer::{IslandInfo, ShapeData, ShapeHash, ShapeSet, meshing::Mesher},
};
use gridmap::hashbrown::hash_map::{Values, ValuesMut};
use petgraph::matrix_graph::{DiMatrix, NodeIndex};

impl<M> Analysis<M> {
    /// Analyze the voxel model
    pub fn build<R>(model: &Model, mesher: &mut Mesher, reader: &mut R) -> Self
    where
        R: QuadsReader<M>,
    {
        // labelize the model
        let mut labeled_model = LabeledModel::from_model(model);

        // identify the shapes of the components and convert them into meshes
        let mut shape_set = ShapeSet::identify_shapes(&mut labeled_model);
        shape_set.build_meshes(mesher, reader);

        // identify connections between islands
        let connectivity = Connectivity::from_model(&labeled_model);

        Self {
            labeled_model,
            shape_set,
            connectivity,
        }
    }
}

impl<M> Analysis<M>
where
    M: Clone,
{
    /// Labeled model
    #[inline]
    pub fn labeled_model(&self) -> &LabeledModel {
        &self.labeled_model
    }

    /// Shape set
    #[inline]
    pub fn shape_set(&self) -> &ShapeSet<M> {
        &self.shape_set
    }

    /// Connectivity of the components
    #[inline]
    pub fn connectivity(&self) -> &Connectivity {
        &self.connectivity
    }

    /// Get the number of components defined
    #[inline]
    pub fn count(&self) -> usize {
        self.labeled_model.islands.len()
    }

    /// Get island informations from a label
    pub fn info_from_label(&self, label: Label) -> Result<Info<'_, M>, Error> {
        let island = self.labeled_model.get_island_info(label)?;
        let mesh = self.shape_set.get_mesh(label)?;
        let node_index = self.connectivity.get_node_index(label)?;
        Ok(Info {
            label,
            voxel_value: island.voxel_value,
            voxels_count: island.voxels_count,
            mesh,
            origin: island.bounding_box.start,
            node_index,
        })
    }

    /// Get island informations from a node index
    pub fn info_from_node_index(&self, node_index: NodeIndex) -> Result<Info<'_, M>, Error> {
        let label = self.connectivity.get_label(node_index)?;
        let island = self.labeled_model.get_island_info(label)?;
        let mesh = self.shape_set.get_mesh(label)?;
        Ok(Info {
            label,
            voxel_value: island.voxel_value,
            voxels_count: island.voxels_count,
            mesh,
            origin: island.bounding_box.start,
            node_index,
        })
    }
}

impl LabeledModel {
    /// Get the info on the island from the label
    pub fn get_island_info(&self, label: Label) -> Result<&IslandInfo, Error> {
        if let Some(island) = self.islands.get(&label) {
            Ok(island)
        } else {
            Err(Error::NoIsland)
        }
    }
}

impl<M> ShapeSet<M> {
    /// Get the shape from the label of the island
    pub fn get_shape(&self, label: Label) -> Result<&ShapeData, Error> {
        if let Some(shape) = self
            .mapping
            .get(&label)
            .and_then(|hash| self.shapes.get(hash))
        {
            Ok(shape)
        } else {
            Err(Error::NoShape)
        }
    }

    /// Get the mesh from the label of the island
    pub fn get_mesh(&self, label: Label) -> Result<&M, Error> {
        if let Some(mesh) = self
            .mapping
            .get(&label)
            .and_then(|hash| self.meshes.get(hash))
        {
            Ok(mesh)
        } else {
            Err(Error::NoMesh)
        }
    }

    /// Access the list of meshes generated
    pub fn get_meshes(&self) -> Values<'_, ShapeHash, M> {
        self.meshes.values()
    }

    /// Access the list of meshes generated
    pub fn get_meshes_mut(&mut self) -> ValuesMut<'_, ShapeHash, M> {
        self.meshes.values_mut()
    }
}

impl Connectivity {
    /// Access the incidence matrix
    #[inline]
    pub fn incidence_matrix(&self) -> &DiMatrix<Label, usize> {
        &self.graph
    }

    /// Get a node index from a label
    pub fn get_node_index(&self, label: Label) -> Result<NodeIndex, Error> {
        if let Some(node_index) = self.mapping.get(&label) {
            Ok(*node_index)
        } else {
            Err(Error::NotConnected)
        }
    }

    /// Get the label from a node index
    pub fn get_label(&self, node_index: NodeIndex) -> Result<Label, Error> {
        if let Some(label) = self.graph.get_node_weight(node_index) {
            Ok(*label)
        } else {
            Err(Error::NoLabel)
        }
    }
}
