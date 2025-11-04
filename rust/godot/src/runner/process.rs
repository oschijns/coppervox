use super::Runner;

impl Runner {
    /// Run a tick of the simulation
    pub fn tick(&mut self) {
        // Select the work buffer to use for this tick
        let buffer = if self.flip_flop {
            &self.work_buffers[1]
        } else {
            &self.work_buffers[0]
        };

        // create a new compute list
        if let Some(mut render) = self.render_device.clone() {
            let list = render.compute_list_begin();
            render.compute_list_bind_compute_pipeline(list, self.pipeline);
            render.compute_list_bind_uniform_set(list, buffer.uniform_set, 0);
            render.compute_list_dispatch(list, self.dispatches_count, 1, 1);
            render.compute_list_end();

            // submit to the GPU and wait for the result
            render.submit();
            render.sync();

            // flip & flop
            self.flip_flop = !self.flip_flop;
        }
    }
}
