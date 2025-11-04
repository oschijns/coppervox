use super::{Buffer, Runner, SamplerBuffer, WorkBuffer, config::*};
use godot::{
    classes::{
        RdTextureFormat, RdTextureView, RdUniform, RenderingDevice,
        rendering_device::{
            BufferCreationBits, DataFormat, TextureSamples, TextureType, TextureUsageBits,
            UniformType,
        },
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
            self.sampler_buffer.build(render.clone(), count);

            // prepare the two work buffers
            let [buffer_a, buffer_b] = &mut self.work_buffers;
            WorkBuffer::build(buffer_a, buffer_b, render.clone(), count);

            // Define two uniform set for swapping between the two work buffers
            buffer_a.build_uniform_set(
                render.clone(),
                &self.buffer_size,
                &self.buffer_connections,
                &self.buffer_operations,
                &self.sampler_buffer,
                self.shader,
            );
            buffer_b.build_uniform_set(
                render.clone(),
                &self.buffer_size,
                &self.buffer_connections,
                &self.buffer_operations,
                &self.sampler_buffer,
                self.shader,
            );

            Ok(())
        } else {
            Err(())
        }
    }

    /// Access the texture handle
    #[inline]
    pub fn get_texture_handle(&self) -> Rid {
        self.sampler_buffer.handle
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

impl Buffer {
    /// Create the buffer with the provided data
    fn build(&mut self, mut render: Gd<RenderingDevice>, integers: &[i32], binding: i32) {
        // Create a storage buffer given the provided data
        let bytes = PackedInt32Array::from(integers).to_byte_array();
        self.handle = render
            .storage_buffer_create_ex(bytes.len() as u32)
            .data(&bytes)
            .done();
        self.uniform.set_binding(binding);
        self.uniform.add_id(self.handle);
    }

    /// Delete the storage buffer associated
    fn delete(&mut self, mut render: Gd<RenderingDevice>) {
        render.free_rid(self.handle);
        self.handle = Rid::Invalid;
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
            uniform_set: Rid::Invalid,
        }
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

    /// Create uniform set
    fn build_uniform_set(
        &mut self,
        mut render: Gd<RenderingDevice>,
        buffer_size: &Buffer,
        buffer_connections: &Buffer,
        buffer_operations: &Buffer,
        sampler_buffer: &SamplerBuffer,
        shader: Rid,
    ) {
        self.uniform_set = render.uniform_set_create(
            &array![
                &buffer_size.uniform,
                &buffer_connections.uniform,
                &buffer_operations.uniform,
                &self.uniform_input,
                &self.uniform_output,
                &sampler_buffer.uniform
            ],
            shader,
            0,
        );
    }

    /// Delete the storage buffers associated
    fn delete(&mut self, mut render: Gd<RenderingDevice>) {
        render.free_rid(self.handle);
        render.free_rid(self.uniform_set);
        self.handle = Rid::Invalid;
        self.uniform_set = Rid::Invalid;
    }
}

/// Allocate memory
impl Default for SamplerBuffer {
    fn default() -> Self {
        let mut uniform = RdUniform::new_gd();
        uniform.set_uniform_type(UniformType::IMAGE);
        uniform.set_binding(BIND_SAMPLER);

        Self {
            handle: Rid::Invalid,
            uniform,
            view: RdTextureView::new_gd(),
        }
    }
}

impl SamplerBuffer {
    /// Create the buffer with the appropriate size
    fn build(&mut self, mut render: Gd<RenderingDevice>, count: u32) {
        let mut fmt = RdTextureFormat::new_gd();
        fmt.set_width(count);
        fmt.set_height(1);
        fmt.set_depth(1);
        fmt.set_format(DataFormat::R32_UINT);
        fmt.set_samples(TextureSamples::SAMPLES_1);
        fmt.set_texture_type(TextureType::TYPE_1D);
        fmt.set_usage_bits(TextureUsageBits::SAMPLING_BIT | TextureUsageBits::STORAGE_BIT);

        self.handle = render.texture_create(&fmt, &self.view);
        self.uniform.add_id(self.handle);
    }

    /// Delete the storage buffer associated
    fn delete(&mut self, mut render: Gd<RenderingDevice>) {
        render.free_rid(self.handle);
        self.handle = Rid::Invalid;
    }
}
