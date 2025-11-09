use super::{ComponentType, Qualifier};
use crate::runner::{ActivationData, config::CVOperation};
use bitvec::{bitvec, order::Lsb0};
use coppervox_analyzer::{Analysis, Connectivity, gridmap::hashbrown::HashMap};
use godot::prelude::*;
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

/// Create a buffer for storing operation types
pub struct GeneratedBuffers {
    /// The list of operations for each component
    operations: Vec<i32>,

    /// List of the initial values for the inputs
    inputs: Vec<i32>,

    /// For a given action, which components are activated
    pub activators: HashMap<StringName, ActivationData>,
}

impl GeneratedBuffers {
    /// Identify the type of each component from the data being read initially
    pub fn build<M>(analysis: &Analysis<M>, qualifier: Gd<Qualifier>) -> Self
    where
        M: Clone,
    {
        // get map from qualifier
        let map = &qualifier.bind().evaluated;

        // The buffers to generate
        let count = analysis.count();
        let mut operations = Vec::<i32>::with_capacity(count);
        let mut inputs = Vec::<i32>::with_capacity(count);
        let mut activators = HashMap::with_capacity(map.len());

        // iterate over each node
        for node_index in analysis
            .connectivity()
            .incidence_matrix()
            .node_identifiers()
        {
            // For the given node, get the associated voxel value
            if let Ok(info) = analysis.info_from_node_index(node_index)
                && let Some(component_type) = map.get(&info.voxel_value)
            {
                match component_type {
                    ComponentType::Constant(value) => {
                        operations.push(CVOperation::Const as i32);
                        inputs.push(*value as i32);
                    }
                    ComponentType::Operation(op) => {
                        operations.push(*op as i32);
                        inputs.push(0);
                    }
                    ComponentType::Action { value, action } => {
                        operations.push(CVOperation::Const as i32);
                        inputs.push(*value as i32);

                        // Register the activator
                        activators
                            .entry(action.clone())
                            .or_insert_with(|| ActivationData::new(*value))
                            .add_node_index(node_index);
                    }
                }
            } else {
                operations.push(CVOperation::Const as i32);
                inputs.push(0);
            }
        }

        Self {
            operations,
            inputs,
            activators,
        }
    }

    /// Get the operations buffer
    #[inline]
    pub fn operations(&self) -> &[i32] {
        &self.operations
    }

    /// Get the inputs buffer
    #[inline]
    pub fn inputs(&self) -> &[i32] {
        &self.inputs
    }
}
