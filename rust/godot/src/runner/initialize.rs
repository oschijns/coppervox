use super::{Buffer, Runner, WorkBuffer, config::*};
use godot::{
    classes::{
        RdUniform, RenderingDevice,
        rendering_device::{BufferCreationBits, UniformType},
    },
    prelude::*,
};

impl Runner {
    /// Build a runner from a set of buffer
    pub fn build(&mut self, count: u32, connections: &[i32], operations: &[i32]) -> Result<(), ()> {
        // compute the number of dispatches necessary
        self.dispatches_count = count.div_ceil(WORKGROUP_SIZE);

        // prepare the storage buffers
        if let Some(render) = self.render_device.clone() {
            self.buffer_size
                .build(render.clone(), &[count as i32], BIND_SIZE);
            self.buffer_connections
                .build(render.clone(), connections, BIND_CONNECTIONS);
            self.buffer_operations
                .build(render.clone(), operations, BIND_OPERATIONS);

            // prepare the two work buffers
            let [buffer_a, buffer_b] = &mut self.work_buffers;
            WorkBuffer::build(buffer_a, buffer_b, render, count);
            Ok(())
        } else {
            Err(())
        }
    }
}

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
    fn build(&mut self, mut render: Gd<RenderingDevice>, integers: &[i32], binding: i32) {
        // Create a storage buffer given the provided data
        let bytes = PackedInt32Array::from(integers).to_byte_array();
        self.handle = render
            .storage_buffer_create_ex(bytes.len() as u32)
            .data(&bytes)
            .done();
        self.uniform.add_id(self.handle);
        self.uniform.set_binding(binding);
    }

    /// Delete the storage buffer associated
    fn delete(&mut self, mut render: Gd<RenderingDevice>) {
        render.free_rid(self.handle);
        self.handle = Rid::Invalid;
    }
}

impl WorkBuffer {
    /// Create the work buffers
    fn build(
        buffer_a: &mut Self,
        buffer_b: &mut Self,
        mut render: Gd<RenderingDevice>,
        count: u32,
    ) {
        // Create a storage buffer with the given size
        let size = count * size_of::<i32>() as u32;
        macro_rules! make {
            () => {
                render
                    .storage_buffer_create_ex(size)
                    .creation_bits(BufferCreationBits::AS_STORAGE_BIT)
                    .done()
            };
        }
        buffer_a.handle = make!();
        buffer_b.handle = make!();

        // link the two buffers as input and output
        macro_rules! set {
            ($buf_in:ident -> $buf_out:ident) => {
                $buf_out.uniform_input.add_id($buf_in.handle);
                $buf_out.uniform_output.add_id($buf_out.handle);
            };
        }
        set! { buffer_a -> buffer_b }
        set! { buffer_b -> buffer_a }
    }

    /// Delete the storage buffers associated
    fn delete(&mut self, mut render: Gd<RenderingDevice>) {
        render.free_rid(self.handle);
        render.free_rid(self.uniforms_list);
        self.handle = Rid::Invalid;
        self.uniforms_list = Rid::Invalid;
    }
}
