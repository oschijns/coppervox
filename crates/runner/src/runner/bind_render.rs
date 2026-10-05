use super::*;
use crate::shader_const::{GpuLayout, GpuMat4x4};
use kiss3d::prelude::*;
use std::num::NonZero;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, PipelineLayoutDescriptor,
    ShaderStages,
};

/// The singular projection matrix of the camera
pub const BIND_PROJECTION: GpuLayout = GpuLayout::new(0, 0);

/// Instance data (translation & color index) of the components
pub const BIND_INSTANCES_DATA: GpuLayout = GpuLayout::new(1, 0);

/// Color presets
pub const BIND_COLOR_PRESETS: GpuLayout = GpuLayout::new(1, 1);

/// Processed values to display on the components
pub const BIND_PROCESSED: GpuLayout = GpuLayout::new(2, 0);

impl RenderLayout {
    /// Create the layouts for the compute shader
    pub fn new(ctx: &Context) -> Self {
        Self {
            layout_proj: create_proj_layout(ctx),
            layout_static: create_static_layout(ctx),
            layout_process: create_process_layout(ctx),
        }
    }
}

/// Load the render shader and build the pipeline used to draw the circuit,
/// specialized lazily per MSAA sample count (only known when `render()` runs).
pub fn create_render_pipeline(ctx: &Context, layout: &RenderLayout) -> PipelineCache {
    let shader = ctx.create_shader_module(
        Some("logic_render_shader"),
        include_str!("../../shaders/render.wgsl"),
    );
    let pipeline_layout = ctx.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("logic_render_pipeline_layout"),
        bind_group_layouts: &[
            Some(&layout.layout_proj),
            Some(&layout.layout_static),
            Some(&layout.layout_process),
        ],
        immediate_size: 0,
    });

    // Three separate vertex buffers: positions, normals, and the UV attribute
    // that is hijacked to carry two `u32` indices (see `render.wgsl`).
    let vertex_buffer_layouts = [
        wgpu::VertexBufferLayout {
            array_stride: size_of::<[f32; 3]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: size_of::<[f32; 3]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            }],
        },
        wgpu::VertexBufferLayout {
            array_stride: size_of::<[f32; 2]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 2,
                format: wgpu::VertexFormat::Uint32x2,
            }],
        },
    ];

    PipelineCache::new(move |sample_count| {
        let ctx = Context::get();
        ctx.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("logic_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                buffers: &vertex_buffer_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // The scene is rendered into the linear HDR film (resolved to
                    // the surface by the tonemap pass), not the surface format.
                    format: Context::render_format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Context::depth_format(),
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            // Must match the scene's MSAA sample count or the pipeline is
            // incompatible with the render pass.
            multisample: multisample_state(sample_count),
            multiview_mask: None,
            cache: None,
        })
    })
}

impl RenderBinding {
    /// Create the binding based on the buffer and layout that have been generated
    pub fn new(ctx: &Context, layout: &RenderLayout, buffers: &AllocBuffers) -> Self {
        let [ping, pong] = &buffers.ping_pong;
        Self {
            bind_proj: create_proj_bind(ctx, &layout.layout_proj, &buffers.projection),
            bind_static: create_static_bind(
                ctx,
                &layout.layout_static,
                &buffers.instances_data,
                &buffers.color_presets,
            ),
            ping_pong: [
                create_process_bind(ctx, "ping", &layout.layout_process, pong),
                create_process_bind(ctx, "pong", &layout.layout_process, ping),
            ],
        }
    }
}

/// Create a bind group layout for the projection matrix, read by `render.wgsl`'s vertex stage
fn create_proj_layout(ctx: &Context) -> BindGroupLayout {
    ctx.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("projection"),
        entries: &[BindGroupLayoutEntry {
            binding: BIND_PROJECTION.binding,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZero::new(size_of::<GpuMat4x4>() as u64),
            },
            count: None,
        }],
    })
}

/// Create a bind group layout for the instance/color buffers, read by `render.wgsl`'s vertex stage
fn create_static_layout(ctx: &Context) -> BindGroupLayout {
    ctx.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("statics"),
        entries: &[
            BindGroupLayoutEntry {
                binding: BIND_INSTANCES_DATA.binding,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: BIND_COLOR_PRESETS.binding,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

/// Create a bind group layout for the two processed ping-pong buffers
fn create_process_layout(ctx: &Context) -> BindGroupLayout {
    ctx.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("processed"),
        entries: &[BindGroupLayoutEntry {
            binding: BIND_PROCESSED.binding,
            visibility: ShaderStages::COMPUTE | ShaderStages::VERTEX_FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

/// Create a bind group for the constant storage buffers
fn create_proj_bind(ctx: &Context, layout: &BindGroupLayout, projection: &Buffer) -> BindGroup {
    ctx.create_bind_group(&BindGroupDescriptor {
        label: Some("projection"),
        layout,
        entries: &[BindGroupEntry {
            binding: BIND_PROJECTION.binding,
            resource: projection.as_entire_binding(),
        }],
    })
}

/// Create a bind group for the static storage buffers containing
fn create_static_bind(
    ctx: &Context,
    layout: &BindGroupLayout,
    instances_data: &Buffer,
    color_presets: &Buffer,
) -> BindGroup {
    ctx.create_bind_group(&BindGroupDescriptor {
        label: Some("statics"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: BIND_INSTANCES_DATA.binding,
                resource: instances_data.as_entire_binding(),
            },
            BindGroupEntry {
                binding: BIND_COLOR_PRESETS.binding,
                resource: color_presets.as_entire_binding(),
            },
        ],
    })
}

/// Create a bind group for the processed ping-pong buffers
fn create_process_bind(
    ctx: &Context,
    label: &str,
    layout: &BindGroupLayout,
    buffer: &Buffer,
) -> BindGroup {
    ctx.create_bind_group(&BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[BindGroupEntry {
            binding: BIND_PROCESSED.binding,
            resource: buffer.as_entire_binding(),
        }],
    })
}
