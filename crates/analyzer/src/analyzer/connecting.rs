use super::{Connectivity, Label, LabeledModel};
use coppervox_common::gridmap::hashbrown::HashMap;
use petgraph::matrix_graph::{DiMatrix, NodeIndex};
use std::cmp::max;

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
                    *occurences.entry(neigh).or_insert(1) += 1;
                }
            }

            // Now we can generate edges coming from the neighbors toward this cell.
            for (neigh, &occ) in occurences.iter() {
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
