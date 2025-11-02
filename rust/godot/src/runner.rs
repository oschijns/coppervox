/// Setup the runner
mod initialize;

/// Run the runner
mod process;

use godot::{classes::RdUniform, prelude::*};

/// CopperVox runner for the logic circuit
#[derive(GodotClass)]
#[class(base=Node)]
pub struct CvRunner {
    base: Base<Node>,

    /// The number of components to operate
    count: usize,

    /// Pipeline for running the compute shader
    pipeline: Rid,

    /// Storage buffer for the size
    buffer_size: Buffer,

    /// Storage buffer for the connections
    buffer_connections: Buffer,

    /// Storage buffer for the type of operations
    buffer_operations: Buffer,

    /// IO work buffers to swap every tick
    work_buffers: [WorkBuffer; 2],
}

/// Keep track of the handle to the storage buffer and the uniform associated with it
struct Buffer {
    /// handle to the storage buffer
    handle: Rid,

    /// keep track of the uniform built
    uniform: Gd<RdUniform>,
}

/// Handles for a work buffer
struct WorkBuffer {
    /// Handle to the storage buffer
    handle: Rid,

    /// keep track of the uniform built as an input
    uniform_input: Gd<RdUniform>,

    /// keep track of the uniform built as an output
    uniform_output: Gd<RdUniform>,

    /// Uniforms list for using the storage buffer as an output
    uniforms_list: Rid,
}

#[godot_api]
impl INode for CvRunner {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            count: 0,
            pipeline: Rid::Invalid,
            buffer_size: Default::default(),
            buffer_connections: Default::default(),
            buffer_operations: Default::default(),
            work_buffers: Default::default(),
        }
    }

    fn ready(&mut self) {}
}

#[godot_api]
impl CvRunner {}
