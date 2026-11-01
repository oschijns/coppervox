use super::{Connectivity, Label, LabeledModel};
use coppervox_common::gridmap::hashbrown::HashMap;
use petgraph::matrix_graph::{DiMatrix, NodeIndex};
use std::cmp::max;

/// Minimum number of face-adjacent voxels required to form a directed connection.
const MIN_CONNECTION_NEIGHBORS: usize = 3;

impl Connectivity {
    /// Identify connections between islands
    pub fn from_model(model: &LabeledModel) -> Self {
        let mut graph = DiMatrix::with_capacity(model.islands.len());

        // generate a node for each label
        let mapping: HashMap<Label, NodeIndex> = model
            .islands
            .keys()
            .map(|label| (*label, graph.add_node(*label)))
            .collect();

        // Buffer for counting the number of occurences of each label
        // encountered when checking for neighbors.
        let mut occurences = HashMap::<Label, usize>::with_capacity(6);

        // iterate over the whole matrix and connect nodes according to
        for ([x, y, z], label) in model.labeled_model.indexed_iter() {
            // Skip empty cells, they don't belong to any island.
            if *label == 0 {
                continue;
            }

            // Check the cells around the current one.
            // TODO: This could be better optimized by using a smarter iterator.
            let neighbors = [
                model.labeled_model[[x - 1, y, z]],
                model.labeled_model[[x + 1, y, z]],
                model.labeled_model[[x, y - 1, z]],
                model.labeled_model[[x, y + 1, z]],
                model.labeled_model[[x, y, z - 1]],
                model.labeled_model[[x, y, z + 1]],
            ];

            // Check the occurences of each individual label in the direct
            // neighbors of the current cell.
            occurences.clear();
            for neigh in neighbors {
                // Skip if the neighbor is empty space (label 0) or part of
                // the same island as the current cell.
                if neigh != 0 && neigh != *label {
                    *occurences.entry(neigh).or_insert(0) += 1;
                }
            }

            // Now we can generate edges coming from the neighbors toward this cell.
            for (neigh, &occ) in occurences.iter() {
                if occ < MIN_CONNECTION_NEIGHBORS {
                    continue;
                }

                // we have identified that we want to draw an edge between two nodes
                let node_target = mapping[label];
                let node_origin = mapping[neigh];

                // Check if that edge already exists, and update its value.
                if let Some(edge) = graph.get_edge_weight_mut(node_origin, node_target) {
                    // Keep the maximum only.
                    *edge = max(*edge, occ);
                } else {
                    let _ = graph.update_edge(node_origin, node_target, occ);
                }
            }
        }

        // Return the generated graph
        Self { mapping, graph }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::LabeledModel;
    use coppervox_common::Model;

    fn connectivity_with_neighbor_count(count: usize) -> (Connectivity, Label, Label) {
        let mut model = Model::new([12, 12, 12]);
        model[[0, 0, 0]] = 1;

        // The ring keeps all surrounding voxels in one component while
        // controlling how many of them touch the center voxel.
        for x in -1..=1 {
            for y in -1..=1 {
                if (x != 0 || y != 0) && !(count == 3 && [x, y] == [1, 0]) {
                    model[[x, y, 0]] = 2;
                }
            }
        }

        let labeled = LabeledModel::from_model(&model);
        let target = labeled
            .islands
            .iter()
            .find_map(|(&label, island)| (island.voxel_value() == 1).then_some(label))
            .expect("target component should be present");
        let source = labeled
            .islands
            .iter()
            .find_map(|(&label, island)| (island.voxel_value() == 2).then_some(label))
            .expect("source component should be present");

        (Connectivity::from_model(&labeled), source, target)
    }

    #[test]
    fn requires_more_than_three_neighbor_voxels_for_a_connection() {
        let (connectivity, source, target) = connectivity_with_neighbor_count(3);
        let source_node = connectivity.mapping[&source];
        let target_node = connectivity.mapping[&target];

        assert_eq!(
            connectivity.graph.get_edge_weight(source_node, target_node),
            Some(&3)
        );
    }

    #[test]
    fn connects_components_when_a_voxel_has_four_matching_neighbors() {
        let (connectivity, source, target) = connectivity_with_neighbor_count(4);
        let source_node = connectivity.mapping[&source];
        let target_node = connectivity.mapping[&target];

        assert_eq!(
            connectivity.graph.get_edge_weight(source_node, target_node),
            Some(&4)
        );
        assert_eq!(
            connectivity.graph.get_edge_weight(target_node, source_node),
            None
        );
    }
}
