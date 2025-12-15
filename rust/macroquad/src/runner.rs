/// Compute new values for each component
pub mod process;

/// Define the operations
pub mod operations;

/// Render the logic circuit
pub mod render;

use crate::base::{Colorize, CompType, Value};
use macroquad::{
    input::KeyCode,
    math::{Mat4, Vec3},
    models::Mesh,
};
use petgraph::graph::DiGraph;
use std::sync::Arc;

/// Store data for running a logic circuit simulation
pub struct Runner {
    /// Store the logic circuit as a graph
    graph: DiGraph<Component, ()>,

    /// Buffer for storing values to be processed
    buffer: Vec<Value>,
}

/// Component of the simulation
pub struct Component {
    /// Current value to compute
    current_value: Value,

    /// Previous value
    previous_value: Value,

    /// Resolved operation
    operand: Operand,

    /// Define how the component should be colorized
    colorize: Arc<Colorize>,

    /// Transform to position the mesh in the scene
    transform: Mat4,

    /// Mesh to be rendered
    mesh: Arc<Mesh>,
}

/// Read inputs and compute a new current value
pub enum Operand {
    /// Always output the same value
    Const(Value),

    /// Read a user's input and output a value
    Input {
        /// Read an input from the user
        input: KeyCode,

        /// Output this value if the input is pressed
        value: Value,
    },

    /// Read input values and output a result
    Gate(Box<fn(values: &[Value]) -> Value>),
}

impl Runner {
    /// Create a new runner
    #[inline]
    pub fn new(graph: DiGraph<Component, ()>) -> Self {
        let buffer = Vec::with_capacity(graph.node_count());
        Self { graph, buffer }
    }
}

impl Component {
    /// Create a new component to render
    #[inline]
    pub fn new(
        comp_type: CompType,
        colorize: Arc<Colorize>,
        position: Vec3,
        mesh: Arc<Mesh>,
    ) -> Self {
        // If the value of the component is constant, return said value
        let value = if let CompType::Const(value) = comp_type {
            value
        } else {
            0
        };

        // Build the component
        Self {
            current_value: value,
            previous_value: value,
            operand: comp_type.into(),
            colorize,
            transform: Mat4::from_translation(position),
            mesh,
        }
    }
}
