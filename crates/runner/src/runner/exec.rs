use crate::runner::{
    AllocBuffers, ComputeBinding, ComputeLayout, RenderBinding, RenderLayout, Runner,
    RunnerGpuData,
    bind_render::{BIND_INSTANCES_DATA, BIND_PROCESSED, BIND_PROJECTION},
};
use kiss3d::{
    camera::Camera3d,
    context::Context,
    glamx::{Pose3, Vec3},
    light::LightCollection,
    resource::{GpuData, GpuMesh3d, Material3d, RenderContext, vertex_index::VERTEX_INDEX_FORMAT},
    scene::{InstancesBuffer3d, ObjectData3d},
};
use std::any::Any;

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

    /// Prepare the material (if necessary?)
    fn prepare(
        &mut self,
        pass: usize,
        transform: Pose3,
        scale: Vec3,
        camera: &mut dyn Camera3d,
        lights: &LightCollection,
        data: &ObjectData3d,
        gpu_data: &mut dyn GpuData,
        viewport_width: u32,
        viewport_height: u32,
    ) {
        let ctx = Context::get();
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

        // Switch between the two buffers
        let flip_flop = if self.flip_flop { 1 } else { 0 };
        self.flip_flop = !self.flip_flop;

        // Prepare the pipeline to render the circuit
        let pipeline = self.pipeline.get(context.sample_count);
        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(BIND_PROJECTION.group, &gpu_data.render.bind_proj, &[]);
        render_pass.set_bind_group(BIND_INSTANCES_DATA.group, &gpu_data.render.bind_static, &[]);
        render_pass.set_bind_group(
            BIND_PROCESSED.group,
            &gpu_data.render.ping_pong[flip_flop],
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
        let compute = ComputeBinding::new(ctx, &layout_compute, &buffers);
        let render = RenderBinding::new(ctx, &layout_render, &buffers);
        Self {
            buffers,
            compute,
            render,
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
