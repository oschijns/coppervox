use super::{Analysis, Error, IslandInfo, ShapeHash};
use crate::{Info, Label};
use gridmap::hashbrown::{HashMap, hash_map};
use petgraph::matrix_graph::NodeIndex;

/// Iterator for reading the result of an analysis
pub struct Iter<'a, M> {
    /// Main iterator to iterate over
    main_iter: hash_map::Iter<'a, Label, IslandInfo>,

    /// Provide access to generated mesh
    meshes: &'a HashMap<ShapeHash, M>,

    /// Provide access to graph nodes
    nodes: &'a HashMap<Label, NodeIndex>,

    /// List of incomplete labels to report as error
    invalid_labels: Option<Vec<(Label, Error)>>,
}

impl<M> Analysis<M>
where
    M: Clone,
{
    pub fn iter<'a>(&'a self) -> Iter<'a, M> {
        Iter {
            main_iter: self.labeled_model.islands.iter(),
            meshes: &self.meshes,
            nodes: &self.connectivity.mapping,
            invalid_labels: None,
        }
    }
}

impl<M> Iter<'_, M>
where
    M: Clone,
{
    /// Add error reporting to this iterator
    #[inline]
    pub fn report_errors(&mut self) -> &Self {
        let size = self.main_iter.len();
        self.invalid_labels = Some(Vec::with_capacity(size));
        self
    }
}

impl<M> Iterator for Iter<'_, M>
where
    M: Clone,
{
    type Item = (Label, Info<M>);

    /// Get next label with its data
    fn next(&mut self) -> Option<Self::Item> {
        // Report error encountered is the option is activated
        macro_rules! report {
            ($label:ident -> $error:ident) => {
                if let Some(list) = &mut self.invalid_labels {
                    list.push((*$label, Error::$error));
                }
            };
        }

        // Try to get the next complete island information
        while let Some((label, island)) = self.main_iter.next() {
            // Get the associated node index
            let Some(&node_index) = self.nodes.get(label) else {
                report!(label -> NotConnected);
                continue;
            };

            // Get the associated mesh
            let Some(mesh) = island
                .shape_hash
                .and_then(|hash| self.meshes.get(&hash).cloned())
            else {
                report!(label -> NoMesh);
                continue;
            };

            // We found a complete island, return it
            return Some((
                *label,
                Info {
                    voxel_value: island.voxel_value,
                    voxels_count: island.voxels_count,
                    mesh,
                    node_index,
                },
            ));
        }
        None
    }
}
