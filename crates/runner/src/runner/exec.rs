use crate::{
    runner::{
        AllocBuffers, ComputeBinding, ComputeLayout, RenderBinding, RenderLayout, Runner,
        RunnerGpuData,
        bind_compute::{self, BIND_CONNECTIONS, BIND_INPUTS},
        bind_render::{self, BIND_INSTANCES_DATA, BIND_PROCESSED, BIND_PROJECTION},
    },
    shader_const::WORKGROUP,
};
use kiss3d::{
    camera::Camera3d,
    context::Context,
    glamx::{Pose3, Vec3},
    light::LightCollection,
    resource::{GpuData, GpuMesh3d, Material3d, RenderContext, vertex_index::VERTEX_INDEX_FORMAT},
    scene::{InstancesBuffer3d, ObjectData3d},
};
use std::{any::Any, time::Instant};

impl Material3d for Runner {
    /// Create the GPU data
    fn create_gpu_data(&self) -> Box<dyn GpuData> {
        let ctx = Context::get();
        let buffers = AllocBuffers::new(
            &ctx,
            &self.data.projection,
            &self.data.connections,
            &self.data.operations,
            &self.data.instances_data,
            &self.data.color_presets,
            &self.data.inits,
        );
        Box::new(RunnerGpuData::new(&ctx, buffers))
    }

    /// Run configured simulation ticks before the scene is drawn
    fn prepare(
        &mut self,
        pass: usize,
        _transform: Pose3,
        _scale: Vec3,
        camera: &mut dyn Camera3d,
        _lights: &LightCollection,
        _data: &ObjectData3d,
        gpu_data: &mut dyn GpuData,
        _viewport_width: u32,
        _viewport_height: u32,
    ) {
        // A stereo camera calls `prepare` once per eye; only tick the
        // simulation once per frame, on the first pass.
        if pass != 0 {
            return;
        }

        let ctx = Context::get();

        // Downcast gpu_data to our specific type
        let gpu_data = gpu_data
            .as_any_mut()
            .downcast_mut::<RunnerGpuData>()
            .expect("Runner requires RunnerGpuData");

        // Track the moving camera. `render.wgsl` expects a single combined
        // view-projection matrix: there is no per-object model matrix, since
        // instance translations are applied directly to the vertex position.
        let (view, proj) = camera.view_transform_pair(pass);
        let view_proj = proj * view.to_mat4();
        ctx.write_buffer(
            &gpu_data.buffers.projection,
            0,
            bytemuck::bytes_of(&view_proj.to_cols_array_2d()),
        );

        let now = Instant::now();
        self.tick_accumulator += now.duration_since(self.last_tick);
        self.last_tick = now;

        let mut ticks = 0;
        while self.tick_accumulator >= self.tick_interval {
            self.tick_accumulator -= self.tick_interval;
            ticks += 1;
        }
        if ticks == 0 {
            return;
        }

        let pressed_keys = self.pressed_keys.borrow();
        for binding in self.input_bindings.iter() {
            let value = if pressed_keys.contains(&binding.key) {
                binding.value
            } else {
                0
            };
            let offset = (binding.component * size_of::<u32>()) as u64;
            let bytes = bytemuck::bytes_of(&value);
            for buffer in &gpu_data.buffers.ping_pong {
                ctx.write_buffer(buffer, offset, bytes);
            }
        }

        // The render pass encoder used by kiss3d isn't handed to `prepare`, so
        // record and submit compute ticks on our own command buffer. It is
        // submitted before kiss3d's frame encoder, ensuring rendering sees the
        // latest completed tick.
        let mut encoder = ctx.create_command_encoder(Some("logic_compute"));
        let count = self.data.operations.len() as u32;
        let workgroups = count.div_ceil(WORKGROUP);
        for _ in 0..ticks {
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("logic_compute_pass"),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&gpu_data.compute_pipeline);
                compute_pass.set_bind_group(
                    BIND_CONNECTIONS.group,
                    &gpu_data.compute.bind_const,
                    &[],
                );
                compute_pass.set_bind_group(
                    BIND_INPUTS.group,
                    &gpu_data.compute.ping_pong[self.current_buffer],
                    &[],
                );
                compute_pass.dispatch_workgroups(workgroups, 1, 1);
            }
            self.current_buffer = 1 - self.current_buffer;
        }
        ctx.submit([encoder.finish()]);
    }

    /// Render the circuit
    fn render(
        &mut self,
        _pass: usize,
        _transform: Pose3,
        _scale: Vec3,
        _camera: &mut dyn Camera3d,
        _lights: &LightCollection,
        data: &ObjectData3d,
        mesh: &mut GpuMesh3d,
        _instances: &mut InstancesBuffer3d,
        gpu_data: &mut dyn GpuData,
        render_pass: &mut wgpu::RenderPass<'_>,
        context: &RenderContext,
    ) {
        if !data.surface_rendering_active() {
            return;
        }

        // Downcast gpu_data to our specific type
        let gpu_data = gpu_data
            .as_any_mut()
            .downcast_mut::<RunnerGpuData>()
            .expect("Runner requires RunnerGpuData");

        // Ensure mesh buffers are on GPU
        mesh.coords().write().unwrap().load_to_gpu();
        mesh.normals().write().unwrap().load_to_gpu();
        mesh.uvs().write().unwrap().load_to_gpu();
        mesh.faces().write().unwrap().load_to_gpu();

        let buffer_coords = mesh.coords().read().unwrap();
        let buffer_normals = mesh.normals().read().unwrap();
        let buffer_uvs = mesh.uvs().read().unwrap();
        let buffer_faces = mesh.faces().read().unwrap();

        let buffer_coords = match buffer_coords.buffer() {
            Some(b) => b,
            None => return,
        };
        let buffer_normals = match buffer_normals.buffer() {
            Some(b) => b,
            None => return,
        };
        let buffer_uvs = match buffer_uvs.buffer() {
            Some(b) => b,
            None => return,
        };
        let buffer_faces = match buffer_faces.buffer() {
            Some(b) => b,
            None => return,
        };

        // Prepare the pipeline to render the circuit
        let pipeline = gpu_data.render_pipeline.get(context.sample_count);
        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(BIND_PROJECTION.group, &gpu_data.render.bind_proj, &[]);
        render_pass.set_bind_group(BIND_INSTANCES_DATA.group, &gpu_data.render.bind_static, &[]);
        render_pass.set_bind_group(
            BIND_PROCESSED.group,
            &gpu_data.render.ping_pong[self.current_buffer],
            &[],
        );

        render_pass.set_vertex_buffer(0, buffer_coords.slice(..));
        render_pass.set_vertex_buffer(1, buffer_normals.slice(..));
        render_pass.set_vertex_buffer(2, buffer_uvs.slice(..));
        render_pass.set_index_buffer(buffer_faces.slice(..), VERTEX_INDEX_FORMAT);

        render_pass.draw_indexed(0..mesh.num_indices(), 0, 0..1);
    }
}

impl RunnerGpuData {
    /// prepare a runner
    pub fn new(ctx: &Context, buffers: AllocBuffers) -> Self {
        let layout_compute = ComputeLayout::new(ctx);
        let layout_render = RenderLayout::new(ctx);
        let compute_pipeline = bind_compute::create_compute_pipeline(ctx, &layout_compute);
        let render_pipeline = bind_render::create_render_pipeline(ctx, &layout_render);
        let compute = ComputeBinding::new(ctx, &layout_compute, &buffers);
        let render = RenderBinding::new(ctx, &layout_render, &buffers);
        Self {
            buffers,
            compute,
            compute_pipeline,
            render,
            render_pipeline,
        }
    }
}

impl GpuData for RunnerGpuData {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
