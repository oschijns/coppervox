use super::{
    Compute,
    config::{BIND_INPUTS, BIND_OUTPUTS, BIND_SIZE},
};
use godot::{
    classes::{RenderingDevice, rendering_device::BufferCreationBits},
    prelude::*,
};

impl Compute {
    /// Run a tick of the simulation
    pub fn tick(&mut self, mut render: Gd<RenderingDevice>) {
        // Create a storage buffer
        let handle_input = render
            .storage_buffer_create_ex(self.work_buffer.len() as u32)
            .data(&self.work_buffer)
            .creation_bits(BufferCreationBits::AS_STORAGE_BIT)
            .done();

        // Assign the new storage buffer to the uniform used for inputs
        self.uniform_inputs.add_id(handle_input);

        // Create a uniform set for input values
        let uniform_set_input = render.clone().uniform_set_create(
            &self.uniform_list_input,
            self.shader,
            BIND_INPUTS.set,
        );

        // create a new compute list
        let list = render.compute_list_begin();
        render.compute_list_bind_compute_pipeline(list, self.pipeline);
        render.compute_list_bind_uniform_set(list, self.uniform_set_static, BIND_SIZE.set);
        render.compute_list_bind_uniform_set(list, uniform_set_input, BIND_INPUTS.set);
        render.compute_list_bind_uniform_set(list, self.uniform_set_output, BIND_OUTPUTS.set);
        render.compute_list_dispatch(list, self.dispatches_count, 1, 1);
        render.compute_list_end();

        // submit to the GPU and wait for the result
        render.submit();
        render.sync();

        // retreive the data computed
        self.work_buffer = render.buffer_get_data(self.buffer_outputs.handle);

        // delete the uniform set for the inputs
        render.free_rid(uniform_set_input);
        render.free_rid(handle_input);
    }
}
