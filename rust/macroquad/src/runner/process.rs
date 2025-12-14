use super::{Component, Runner};
use crate::{base::INIT, runner::Operand};
use macroquad::input::is_key_down;
use petgraph::Direction;

impl Runner {
    /// For each component, compute its next value
    pub fn compute_values(&mut self) {
        // iterate over each node
        for node_index in self.graph.node_indices() {
            if let Some(comp) = self.graph.node_weight(node_index) {
                self.buffer.clear();

                // Read other components values only if necessary
                if comp.operand.read_others() {
                    // check neighbors of this node and accumulate their values into the buffer
                    for neigh_index in self
                        .graph
                        .neighbors_directed(node_index, Direction::Incoming)
                    {
                        if let Some(neigh) = self.graph.node_weight(neigh_index) {
                            // Pick the previous value
                            self.buffer.push(neigh.previous_value);
                        }
                    }
                }

                // Based on the operand, compute the value to store
                let mut result = INIT;
                match &comp.operand {
                    Operand::Gate(op) => {
                        result = op(&self.buffer);
                    }
                    Operand::Const(value) => {
                        result = *value;
                    }
                    Operand::Input { input, value } => {
                        if is_key_down(*input) {
                            result = *value;
                        }
                    }
                }

                // Get the component again as mutable
                let comp = self.graph.node_weight_mut(node_index).unwrap();
                comp.current_value = result;
            }
        }
    }

    /// Swap the values of each component
    pub fn swap_values(&mut self) {
        for component in self.graph.node_weights_mut() {
            component.swap_value();
        }
    }
}

impl Component {
    /// Store the current value in previous field and reset afterward
    #[inline]
    fn swap_value(&mut self) {
        self.previous_value = self.current_value;
        self.current_value = 0;
    }
}

impl Operand {
    /// Specify if this type of operand will read other components' values
    #[inline]
    fn read_others(&self) -> bool {
        match self {
            Self::Gate(_) => true,
            _ => false,
        }
    }
}
