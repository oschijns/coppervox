use super::{Buffer, Compute, Runner, SamplerBuffer, config::*};
use godot::{
    classes::{
        RdShaderFile, RdTextureFormat, RdUniform, RenderingDevice, RenderingServer,
        rendering_device::{
            BufferCreationBits, DataFormat, TextureSamples, TextureType, TextureUsageBits,
            UniformType,
        },
    },
    prelude::*,
};

impl Runner {
    /// Compile the shader
    pub fn compile_shader(&mut self) {
        self.render_device = RenderingServer::singleton().create_local_rendering_device();
        if let Some(render) = self.render_device.clone() {
            self.compute_data.compile_shader(render, &self.shader_path);
        }
    }

    /// Build a runner from a set of buffer
    pub fn build(
        &mut self,
        count: usize,
        connections: &[i32],
        operations: &[i32],
    ) -> Result<(), ()> {
        // prepare the storage buffers
        if let Some(render) = self.render_device.clone() {
            self.buffer_size.build_uniform(
                render.clone(),
                [count as u64, count as u64],
                BIND_SIZE.binding,
            );
            self.buffer_connections.build_storage(
                render.clone(),
                connections,
                BIND_CONNECTIONS.binding,
            );
            self.buffer_operations.build_storage(
                render.clone(),
                operations,
                BIND_OPERATIONS.binding,
            );

            self.sampler_buffer
                .build(render.clone(), count, BIND_SAMPLER.binding);

            self.compute_data.build(
                render,
                count,
                &self.buffer_size.uniform,
                &self.buffer_connections.uniform,
                &self.buffer_operations.uniform,
                &self.sampler_buffer.uniform,
            );

            Ok(())
        } else {
            Err(())
        }
    }

    /// Delete the storage buffers associated
    fn delete(&mut self, render: Gd<RenderingDevice>) {
        self.buffer_size.delete(render.clone());
        self.buffer_connections.delete(render.clone());
        self.buffer_operations.delete(render.clone());
        self.sampler_buffer.delete(render.clone());
        self.compute_data.delete(render);
    }
}

impl Buffer {
    /// Create the buffer with the provided data
    fn build_uniform<const S: usize>(
        &mut self,
        mut render: Gd<RenderingDevice>,
        data: [u64; S],
        binding: i32,
    ) {
        // Create an uniform buffer given the provided data
        let mut bytes = PackedByteArray::new();
        bytes.resize(S);
        for (i, &d) in data.iter().enumerate() {
            let _ = bytes.encode_u64(i * size_of::<u64>(), d);
        }
        self.handle = render
            .uniform_buffer_create_ex(bytes.len() as u32)
            .data(&bytes)
            .done();
        self.uniform.set_uniform_type(UniformType::UNIFORM_BUFFER);
        self.uniform.set_binding(binding);
        self.uniform.add_id(self.handle);
    }

    /// Create the buffer with the provided data
    fn build_storage(&mut self, mut render: Gd<RenderingDevice>, integers: &[i32], binding: i32) {
        // Create a storage buffer given the provided data
        let bytes = PackedInt32Array::from(integers).to_byte_array();
        self.handle = render
            .storage_buffer_create_ex(bytes.len() as u32)
            .data(&bytes)
            .done();
        self.uniform.set_uniform_type(UniformType::STORAGE_BUFFER);
        self.uniform.set_binding(binding);
        self.uniform.add_id(self.handle);
    }

    /// Create the buffer with the provided data
    fn build_read_write_storage(
        &mut self,
        mut render: Gd<RenderingDevice>,
        count: usize,
        binding: i32,
    ) {
        // Create a storage buffer given the provided data
        self.handle = render
            .storage_buffer_create_ex((count * ELEMENT_SIZE) as u32)
            .creation_bits(BufferCreationBits::AS_STORAGE_BIT)
            .done();
        self.uniform.set_uniform_type(UniformType::STORAGE_BUFFER);
        self.uniform.set_binding(binding);
        self.uniform.add_id(self.handle);
    }

    /// Delete the storage buffer associated
    fn delete(&mut self, mut render: Gd<RenderingDevice>) {
        render.free_rid(self.handle);
    }
}

impl SamplerBuffer {
    /// Create the buffer with the appropriate size
    fn build(&mut self, mut render: Gd<RenderingDevice>, count: usize, binding: i32) {
        let mut fmt = RdTextureFormat::new_gd();
        fmt.set_width(count as u32);
        fmt.set_height(1);
        fmt.set_depth(1);
        fmt.set_format(DataFormat::R32_UINT);
        fmt.set_samples(TextureSamples::SAMPLES_1);
        fmt.set_texture_type(TextureType::TYPE_1D);
        fmt.set_usage_bits(TextureUsageBits::SAMPLING_BIT | TextureUsageBits::STORAGE_BIT);

        self.handle = render.texture_create(&fmt, &self.view);
        self.uniform.set_binding(binding);
        self.uniform.set_uniform_type(UniformType::IMAGE);
        self.uniform.add_id(self.handle);
    }

    /// Delete the storage buffer associated
    fn delete(&mut self, mut render: Gd<RenderingDevice>) {
        render.free_rid(self.handle);
    }
}

impl Compute {
    /// Compile the shader
    fn compile_shader(&mut self, mut render: Gd<RenderingDevice>, shader_path: &GString) {
        let shader = load::<RdShaderFile>(shader_path);
        self.shader = render.shader_create_from_spirv(shader.get_spirv().as_ref());
        self.pipeline = render.compute_pipeline_create(self.shader);
    }

    /// Build the uniform set
    fn build(
        &mut self,
        render: Gd<RenderingDevice>,
        count: usize,
        uniform_size: &Gd<RdUniform>,
        uniform_connections: &Gd<RdUniform>,
        uniform_operations: &Gd<RdUniform>,
        uniform_sampler: &Gd<RdUniform>,
    ) {
        // compute the number of dispatches necessary
        self.dispatches_count = count.div_ceil(WORKGROUP_SIZE) as u32;
        self.buffer_outputs.build_read_write_storage(
            render.clone(),
            count as usize,
            BIND_OUTPUTS.binding,
        );

        // build uniform lists to pass to the rendering pipeline
        let uniform_list_static = array![uniform_size, uniform_connections, uniform_operations,];
        let uniform_list_output = array![&self.buffer_outputs.uniform, uniform_sampler];

        // Create the uniform set for static values
        self.uniform_set_static =
            render
                .clone()
                .uniform_set_create(&uniform_list_static, self.shader, BIND_SIZE.set);

        // The uniform set for input values should be created before every tick
        // The uniform set for output values can be created now too
        self.uniform_set_output =
            render
                .clone()
                .uniform_set_create(&uniform_list_output, self.shader, BIND_OUTPUTS.set);

        // Allocate a buffer large enough for each
        self.work_buffer.resize(count * ELEMENT_SIZE);
    }

    /// Delete data that was allocated
    fn delete(&mut self, mut render: Gd<RenderingDevice>) {
        render.free_rid(self.uniform_set_static);
        render.free_rid(self.uniform_set_output);
        render.free_rid(self.pipeline);
        render.free_rid(self.shader);
        self.buffer_outputs.delete(render);
    }
}
