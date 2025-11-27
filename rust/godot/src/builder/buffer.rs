use super::{ComponentType, Qualifier};
use crate::runner::{
    ActivationData,
    config::{CVOperation, MAX_CONNECTIONS, OP_BIT_SHIFT},
};
use coppervox_analyzer::{Analysis, Connectivity, gridmap::hashbrown::HashMap};
use godot::prelude::*;
use petgraph::visit::IntoNodeIdentifiers;

/// Create a buffer for storing operation types and connections
pub struct GeneratedBuffers {
    /// The list of operations for each component
    operations: Vec<i32>,

    /// Specify the index of the inputs connected to each component
    connections: Vec<i32>,

    /// List of the initial values for the inputs
    inputs: Vec<i32>,

    /// For a given action, which components are activated
    pub activators: HashMap<StringName, ActivationData>,
}

impl GeneratedBuffers {
    /// Identify the type of each component from the data being read initially
    pub fn build<M>(
        analysis: &Analysis<M>,
        qualifier: Gd<Qualifier>,
        contact_threshold: usize,
    ) -> Result<Self, ()>
    where
        M: Clone,
    {
        // get map from qualifier
        let map = &qualifier.bind().evaluated;

        // The buffers to generate
        let count = analysis.count();
        let mut operations = Vec::<i32>::with_capacity(count);
        let mut connections = Vec::<i32>::with_capacity(count * MAX_CONNECTIONS);
        let mut inputs = Vec::<i32>::with_capacity(count);
        let mut activators = HashMap::with_capacity(map.len());

        let graph = analysis.connectivity().incidence_matrix();

        // iterate over each output node
        for node_out in graph.node_identifiers() {
            // allocate a buffer to store connections
            let mut conn: [i32; MAX_CONNECTIONS] = [0; MAX_CONNECTIONS];
            let mut count = 0usize;

            // and for each input node
            for node_in in graph.node_identifiers() {
                if let Some(&contacts) = graph.get_edge_weight(node_in, node_out)
                    && contacts > contact_threshold
                {
                    if count >= MAX_CONNECTIONS {
                        return Err(());
                    }

                    conn[count] = node_in.index() as i32;
                    count += 1;
                }
            }
            // add the connections indices to the buffer
            connections.extend_from_slice(&conn);

            // Get the type of the operation and the initial data in the input buffer
            let (op, input) = if let Ok(info) = analysis.info_from_node_index(node_out)
                && let Some(component_type) = map.get(&info.voxel_value)
            {
                match component_type {
                    ComponentType::Constant(value) => (CVOperation::Const, *value),
                    ComponentType::Operation(op) => (*op, 0),
                    ComponentType::Action { value, action } => {
                        // Register the activator
                        activators
                            .entry(action.clone())
                            .or_insert_with(|| ActivationData::new(*value))
                            .add_node_index(node_out);

                        (CVOperation::Const, *value)
                    }
                }
            } else {
                (CVOperation::Const, 0)
            };

            let combine = (count << OP_BIT_SHIFT) & op as usize;
            operations.push(combine as i32);
            inputs.push(input as i32);
        }

        Ok(Self {
            operations,
            connections,
            inputs,
            activators,
        })
    }

    /// Get the operations buffer
    #[inline]
    pub fn operations(&self) -> &[i32] {
        &self.operations
    }

    /// Get the connections buffer
    #[inline]
    pub fn connections(&self) -> &[i32] {
        &self.connections
    }

    /// Get the inputs buffer
    #[inline]
    pub fn inputs(&self) -> &[i32] {
        &self.inputs
    }
}
