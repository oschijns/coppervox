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
use std::{any::Any, sync::mpsc, time::Instant};

#[derive(Debug, thiserror::Error)]
enum WasmTickError {
    #[error(transparent)]
    Runtime(#[from] crate::loader::wasm_module::WasmRunError),

    #[error("GPU readback failed: {0}")]
    GpuPoll(#[from] wgpu::PollError),

    #[error("GPU readback mapping failed: {0}")]
    BufferMap(#[from] wgpu::BufferAsyncError),

    #[error("GPU readback callback disconnected")]
    ReadbackCallback,

    #[error("GPU readback buffer was not allocated")]
    MissingReadbackBuffer,
}

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
        let wasm_output_count = self.wasm.as_ref().map_or(0, |wasm| wasm.outputs.len());
        Box::new(RunnerGpuData::new(&ctx, buffers, wasm_output_count))
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
        if self.wasm.as_ref().is_some_and(|wasm| wasm.failed) {
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

        {
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
        }

        let count = self.data.operations.len() as u32;
        let workgroups = count.div_ceil(WORKGROUP);

        if self.wasm.is_some() {
            for _ in 0..ticks {
                if let Err(error) = self.exchange_wasm_io(&ctx, gpu_data) {
                    eprintln!("WASM simulation stopped: {error}");
                    if let Some(wasm) = self.wasm.as_mut() {
                        wasm.failed = true;
                    }
                    return;
                }

                // Submit each tick separately so the next WASM exchange reads
                // the output produced by this dispatch.
                let mut encoder = ctx.create_command_encoder(Some("logic_compute"));
                encode_compute_tick(
                    &mut encoder,
                    &gpu_data.compute_pipeline,
                    &gpu_data.compute,
                    self.current_buffer,
                    workgroups,
                );
                self.current_buffer = 1 - self.current_buffer;
                ctx.submit([encoder.finish()]);
            }
        } else {
            // The render pass encoder used by kiss3d isn't handed to `prepare`,
            // so submit compute ticks before kiss3d's frame encoder.
            let mut encoder = ctx.create_command_encoder(Some("logic_compute"));
            for _ in 0..ticks {
                encode_compute_tick(
                    &mut encoder,
                    &gpu_data.compute_pipeline,
                    &gpu_data.compute,
                    self.current_buffer,
                    workgroups,
                );
                self.current_buffer = 1 - self.current_buffer;
            }
            ctx.submit([encoder.finish()]);
        }
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

impl Runner {
    fn exchange_wasm_io(
        &mut self,
        ctx: &kiss3d::context::Context,
        gpu_data: &RunnerGpuData,
    ) -> Result<(), WasmTickError> {
        let Some(wasm) = self.wasm.as_mut() else {
            return Ok(());
        };

        if !wasm.initialized {
            wasm.module.init()?;
            wasm.initialized = true;
        }

        if !wasm.outputs.is_empty() {
            let readback = gpu_data
                .wasm_readback
                .as_ref()
                .ok_or(WasmTickError::MissingReadbackBuffer)?;
            let source = &gpu_data.buffers.ping_pong[self.current_buffer];
            let mut encoder = ctx.create_command_encoder(Some("wasm_io_readback"));
            for (index, binding) in wasm.outputs.iter().enumerate() {
                encoder.copy_buffer_to_buffer(
                    source,
                    (binding.component * size_of::<u32>()) as u64,
                    readback,
                    (index * size_of::<u32>()) as u64,
                    size_of::<u32>() as u64,
                );
            }
            ctx.submit([encoder.finish()]);

            let slice = readback.slice(..);
            let (sender, receiver) = mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result);
            });
            ctx.device.poll(wgpu::PollType::wait_indefinitely())?;
            receiver
                .recv()
                .map_err(|_| WasmTickError::ReadbackCallback)??;

            let mapped = slice.get_mapped_range();
            for (index, binding) in wasm.outputs.iter().enumerate() {
                let start = index * size_of::<u32>();
                let value = u32::from_le_bytes(
                    mapped[start..start + size_of::<u32>()]
                        .try_into()
                        .expect("each readback slot contains one u32"),
                );
                wasm.module.write_input_value(binding.slot, value)?;
            }
            drop(mapped);
            readback.unmap();
        }

        wasm.module.write_input()?;
        wasm.module.tick()?;
        wasm.module.read_output()?;
        for binding in wasm.inputs.iter() {
            let value = wasm.module.read_output_value(binding.slot)?;
            let offset = (binding.component * size_of::<u32>()) as u64;
            let bytes = bytemuck::bytes_of(&value);
            for buffer in &gpu_data.buffers.ping_pong {
                ctx.write_buffer(buffer, offset, bytes);
            }
        }
        Ok(())
    }
}

fn encode_compute_tick(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::ComputePipeline,
    compute: &crate::runner::ComputeBinding,
    current_buffer: usize,
    workgroups: u32,
) {
    let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("logic_compute_pass"),
        timestamp_writes: None,
    });
    compute_pass.set_pipeline(pipeline);
    compute_pass.set_bind_group(BIND_CONNECTIONS.group, &compute.bind_const, &[]);
    compute_pass.set_bind_group(BIND_INPUTS.group, &compute.ping_pong[current_buffer], &[]);
    compute_pass.dispatch_workgroups(workgroups, 1, 1);
}

impl RunnerGpuData {
    /// prepare a runner
    pub fn new(ctx: &Context, buffers: AllocBuffers, wasm_output_count: usize) -> Self {
        let layout_compute = ComputeLayout::new(ctx);
        let layout_render = RenderLayout::new(ctx);
        let compute_pipeline = bind_compute::create_compute_pipeline(ctx, &layout_compute);
        let render_pipeline = bind_render::create_render_pipeline(ctx, &layout_render);
        let compute = ComputeBinding::new(ctx, &layout_compute, &buffers);
        let render = RenderBinding::new(ctx, &layout_render, &buffers);
        let wasm_readback = (wasm_output_count > 0).then(|| {
            ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("wasm_io_readback"),
                size: (wasm_output_count * size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
        Self {
            buffers,
            compute,
            compute_pipeline,
            wasm_readback,
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
