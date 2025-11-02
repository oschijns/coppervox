use super::{Buffer, CvRunner, WorkBuffer};
use godot::{
    classes::{
        RdUniform, RenderingDevice,
        rendering_device::{BufferCreationBits, UniformType},
    },
    prelude::*,
};

// binding points for the buffers in the compute shader
pub const BIND_SIZE: i32 = 0;
pub const BIND_CONNECTIONS: i32 = 1;
pub const BIND_OPERATIONS: i32 = 2;
pub const BIND_INPUTS: i32 = 3;
pub const BIND_OUTPUTS: i32 = 4;
pub const BIND_TEXTURE: i32 = 5;

/// Allocate memory
impl Default for Buffer {
    fn default() -> Self {
        let mut uniform = RdUniform::new_gd();
        uniform.set_uniform_type(UniformType::STORAGE_BUFFER);

        Self {
            handle: Rid::Invalid,
            uniform,
        }
    }
}

/// Allocate memory
impl Default for WorkBuffer {
    fn default() -> Self {
        let mut uniform_input = RdUniform::new_gd();
        let mut uniform_output = RdUniform::new_gd();
        uniform_input.set_uniform_type(UniformType::STORAGE_BUFFER);
        uniform_output.set_uniform_type(UniformType::STORAGE_BUFFER);
        uniform_input.set_binding(BIND_INPUTS);
        uniform_output.set_binding(BIND_OUTPUTS);

        Self {
            handle: Rid::Invalid,
            uniform_input,
            uniform_output,
            uniforms_list: Rid::Invalid,
        }
    }
}

impl Buffer {
    /// Create the buffer with the provided data
    pub fn create(&mut self, render: &mut RenderingDevice, integers: &[i32], binding: i32) {
        // Create a storage buffer given the provided data
        let bytes = PackedInt32Array::from(integers).to_byte_array();
        self.handle = render
            .storage_buffer_create_ex(bytes.len() as u32)
            .data(&bytes)
            .done();
        self.uniform.add_id(self.handle);
        self.uniform.set_binding(binding);
    }
}

impl CvRunner {
    /// Create the work buffers
    pub fn create_work_buffers(&mut self, render: &mut RenderingDevice, count: usize) {
        self.count = count;

        // unwrap the buffers
        let [buf_a, buf_b] = &mut self.work_buffers;

        // Create a storage buffer with the given size
        let size = (count * size_of::<i32>()) as u32;
        macro_rules! make {
            () => {
                render
                    .storage_buffer_create_ex(size)
                    .creation_bits(BufferCreationBits::AS_STORAGE_BIT)
                    .done()
            };
        }
        buf_a.handle = make!();
        buf_b.handle = make!();

        // link the two buffers as input and output
        macro_rules! set {
            ($buf_in:ident -> $buf_out:ident) => {
                $buf_out.uniform_input.add_id($buf_in.handle);
                $buf_out.uniform_output.add_id($buf_out.handle);
            };
        }
        set! { buf_a -> buf_b }
        set! { buf_b -> buf_a }
    }
}
