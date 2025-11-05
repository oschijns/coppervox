use bitvec::{bitvec, order::Lsb0};
use coppervox_analyzer::Connectivity;
use petgraph::visit::IntoNodeIdentifiers;

/// Convert the incidence matrix into a buffer that can be submitted to the compute shader.
pub fn incidence_to_buffer(connect: &Connectivity, contact_threshold: usize) -> Vec<i32> {
    let graph = connect.incidence_matrix();
    let count = graph.node_count();
    let capacity = (count * count).div_ceil(size_of::<i32>());

    // The buffer to generate
    let mut result = Vec::<i32>::with_capacity(capacity);

    // Allocate a buffer to store temporary data
    let mut buffer = bitvec![u32, Lsb0; 0; count];

    // for each output node
    for node_out in graph.node_identifiers() {
        buffer.clear();

        // and for each input node
        for node_in in graph.node_identifiers() {
            if let Some(&contacts) = graph.get_edge_weight(node_in, node_out)
                && contacts > contact_threshold
            {
                buffer.push(true);
            } else {
                // no edge or not enough contacts => no connection
                buffer.push(false);
            }
        }

        // Push the buffer into the result
        result.extend(buffer.as_raw_slice().iter().map(|&i| i as i32));
    }

    result
}
