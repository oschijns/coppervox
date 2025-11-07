/// Configuration directtly related to the compute shader
pub(crate) mod config;

/// Setup the runner
mod initialize;

/// Run the runner
mod process;

use godot::{
    classes::{RdTextureView, RdUniform, RenderingDevice},
    prelude::*,
};
use std::str::FromStr;

/// CopperVox runner for the logic circuit
#[derive(GodotClass)]
#[class(base=Node, rename=CVRunner)]
pub struct Runner {
    base: Base<Node>,

    /// Path to the compute shader to load
    #[export]
    shaper_path: GString,

    /// Handle to shader compiled
    shader: Rid,

    /// The rendering device this runner uses
    render_device: Option<Gd<RenderingDevice>>,

    /// Pipeline for running the compute shader
    pipeline: Rid,

    /// Number of dispatches to run the compute shader with
    dispatches_count: u32,

    /// The number of components to operate
    components_count: u32,

    /// Storage buffer for the size
    buffer_size: Buffer,

    /// Storage buffer for the connections
    buffer_connections: Buffer,

    /// Storage buffer for the type of operations
    buffer_operations: Buffer,

    /// Sampler used to pass data from the compute shader to the rendering shader
    sampler_buffer: SamplerBuffer,

    /// IO work buffers to swap every tick
    work_buffers: [WorkBuffer; 2],

    /// flip-flop flag for specifying which buffer to use as input and the other as output
    flip_flop: bool,
}

/// Keep track of the handle to the storage buffer and the uniform associated with it
struct Buffer {
    /// handle to the storage buffer
    handle: Rid,

    /// uniform associated
    uniform: Gd<RdUniform>,
}

/// Handles for a work buffer
struct WorkBuffer {
    /// Handle to the storage buffer
    handle: Rid,

    /// uniform used as an input
    uniform_input: Gd<RdUniform>,

    /// uniform used as an output
    uniform_output: Gd<RdUniform>,

    /// Uniforms list for using the storage buffer as an output
    uniform_set: Rid,
}

/// Texture buffer shared between the compute shader and the visual shader
struct SamplerBuffer {
    /// Handle to the texture buffer
    handle: Rid,

    /// uniform used by the compute shader
    uniform: Gd<RdUniform>,

    /// view over the texture build
    view: Gd<RdTextureView>,
}

#[godot_api]
impl INode for Runner {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            shaper_path: GString::from_str("res://process.glsl").unwrap(),
            shader: Rid::Invalid,
            render_device: None,
            pipeline: Rid::Invalid,
            dispatches_count: 0,
            components_count: 0,
            buffer_size: Default::default(),
            buffer_connections: Default::default(),
            buffer_operations: Default::default(),
            sampler_buffer: Default::default(),
            work_buffers: Default::default(),
            flip_flop: false,
        }
    }
}

#[godot_api]
impl Runner {
    /// To be called at a fixed tick rate
    #[func]
    fn on_tick(&mut self) {
        self.tick();
    }
}
