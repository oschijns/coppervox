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
    shader_path: GString,

    /// The rendering device this runner uses
    render_device: Option<Gd<RenderingDevice>>,

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

    /// Compute data
    compute_data: Compute,
}

/// Keep track of the handle to the storage buffer and the uniform associated with it
struct Buffer {
    /// handle to the storage buffer
    handle: Rid,

    /// uniform associated
    uniform: Gd<RdUniform>,
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

/// Compute data to submit every tick
struct Compute {
    /// Handle to shader compiled
    shader: Rid,

    /// Pipeline for running the compute shader
    pipeline: Rid,

    /// Number of dispatches to run the compute shader with
    dispatches_count: u32,

    /// Uniform for the input buffer
    uniform_inputs: Gd<RdUniform>,

    /// List of uniforms to build an uniform set for the input
    uniform_list_input: Array<Gd<RdUniform>>,

    /// Storage buffer for the outputs
    buffer_outputs: Buffer,

    /// Uniform set for static values in the compute shader
    uniform_set_static: Rid,

    /// Uniform set for output values in the compute shader
    uniform_set_output: Rid,

    /// Buffer that is submitted as an input then retrieved as an output
    work_buffer: PackedByteArray,
}

#[godot_api]
impl INode for Runner {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            shader_path: GString::from_str("res://process.glsl").unwrap(),
            render_device: None,
            components_count: 0,
            buffer_size: Buffer::default(),
            buffer_connections: Buffer::default(),
            buffer_operations: Buffer::default(),
            sampler_buffer: SamplerBuffer::default(),
            compute_data: Compute::default(),
        }
    }
}

#[godot_api]
impl Runner {
    /// To be called at a fixed tick rate
    #[func]
    fn on_tick(&mut self) {
        if let Some(render) = self.render_device.clone() {
            self.compute_data.tick(render);
        }
    }

    /// Access the texture handle
    #[func]
    #[inline]
    pub fn get_texture_handle(&self) -> Rid {
        self.sampler_buffer.handle
    }
}

/// Allocate memory
impl Default for Buffer {
    fn default() -> Self {
        Self {
            handle: Rid::Invalid,
            uniform: RdUniform::new_gd(),
        }
    }
}

/// Allocate memory
impl Default for SamplerBuffer {
    fn default() -> Self {
        Self {
            handle: Rid::Invalid,
            uniform: RdUniform::new_gd(),
            view: RdTextureView::new_gd(),
        }
    }
}

/// Allocate storage for compute data
impl Default for Compute {
    fn default() -> Self {
        let uniform_inputs = RdUniform::new_gd();
        let uniform_list_input = array![&uniform_inputs];

        Self {
            shader: Rid::Invalid,
            pipeline: Rid::Invalid,
            dispatches_count: 0,
            uniform_inputs,
            uniform_list_input,
            buffer_outputs: Buffer::default(),
            uniform_set_static: Rid::Invalid,
            uniform_set_output: Rid::Invalid,
            work_buffer: PackedByteArray::new(),
        }
    }
}
