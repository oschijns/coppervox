use super::Label;
use ndarray::{Array3, ArrayBase, Ix3, RawData};
use petgraph::{
    matrix_graph::{MatrixGraph, NodeIndex},
    Undirected,
};
use std::collections::{HashMap, HashSet};

/// for each labeled island in the matrix, compute the connectivity with other matrices
pub fn connectivity<L: Label, R: RawData>(
    matrix: &ArrayBase<R, Ix3>,
    labels: &HashSet<L>,
) -> MatrixGraph<L, usize, Undirected> {
    // allocate the output graph with undirected edges
    let mut graph = MatrixGraph::<L, usize, Undirected>::with_capacity(labels.len());

    // generate a node for each label
    let label_to_node_index: HashMap<L, NodeIndex> = labels
        .iter()
        .map(|label| (*label, graph.add_node(*label)))
        .collect();

    // create a new matrix to add the node index data
    let mut node_index_matrix =
        Array3::<(L, Option<NodeIndex>)>::from_elem(matrix.dim(), (L::default(), None));

    // copy the initial matrix and add the node index for each cell
    for cell in node_index_matrix.iter_mut() {
        cell.1 = label_to_node_index.get(&cell.0).copied();
    }

    // iterate over the whole matrix and connect nodes according to
    for ((x, y, z), (_, opt_node)) in node_index_matrix.indexed_iter() {
        if let &Some(node) = opt_node {
            // Check the neighbor along the given axis.
            // If a node exists, generate an edge of weight one.
            // If there was already an edge, increment its weight by one.
            macro_rules! process_neighbor {
                ($axis:ident, $indexes:expr) => {
                    if $axis > 0 {
                        if let Some(prev_node) = node_index_matrix[$indexes].1 {
                            let opt_count = graph.update_edge(node, prev_node, 1);
                            if let Some(count) = opt_count {
                                graph.update_edge(node, prev_node, count + 1);
                            }
                        }
                    }
                };
            }

            process_neighbor!(x, [x - 1, y, z]);
            process_neighbor!(y, [x, y - 1, z]);
            process_neighbor!(z, [x, y, z - 1]);
        }
    }

    graph
}
