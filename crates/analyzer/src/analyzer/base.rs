use super::{Analysis, Connectivity, Error, Info, Label, LabeledModel, QuadsReader};
use crate::analyzer::{IslandInfo, ShapeHash, ShapeSet, meshing::Mesher};
use coppervox_common::{Model, Voxel, gridmap::gridmap::bounding_box::BoundingBox};
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

impl<M> Analysis<M> {
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
        let (_, mesh) = self.shape_set.label_to_mesh(label)?;
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
        let (_, mesh) = self.shape_set.label_to_mesh(label)?;
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

impl IslandInfo {
    /// Get the value of the voxel
    #[inline]
    pub fn voxel_value(&self) -> Voxel {
        self.voxel_value
    }

    /// Get the voxel count
    #[inline]
    pub fn voxel_count(&self) -> usize {
        self.voxels_count
    }

    /// Bounding boxes
    #[inline]
    pub fn bounding_box(&self) -> &BoundingBox<3> {
        &self.bounding_box
    }
}

impl<M> ShapeSet<M> {
    /// Get the number of objects defined in this set.
    /// It should be greater than or equal to the number of shapes.
    #[inline]
    pub fn count_objects(&self) -> usize {
        self.mapping.len()
    }

    /// Get the number of shapes defined in this set.
    #[inline]
    pub fn count_shapes(&self) -> usize {
        self.shapes.len()
    }

    /// From a label, get a hash and the corresponding mesh
    pub fn label_to_mesh(&self, label: Label) -> Result<(ShapeHash, &M), Error> {
        if let Some(&hash) = self.mapping.get(&label)
            && let Some(mesh) = self.meshes.get(&hash)
        {
            Ok((hash, mesh))
        } else {
            Err(Error::NoMesh)
        }
    }

    /// Given a shape hash, find all associated labels
    pub fn find_labels(&self, hash: ShapeHash, labels: &mut Vec<Label>) {
        labels.clear();
        for (label, &hash2) in &self.mapping {
            if hash == hash2 {
                labels.push(*label);
            }
        }
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
