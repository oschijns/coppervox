/// Allocate the buffers for the compute and the render passes
mod alloc_buffers;

/// Generate the storage buffers and appropriate binding groups for compute pass
mod bind_compute;

/// Generate the storage buffers and appropriate binding groups for render pass
mod bind_render;

/// Execute the runner
mod exec;

use crate::shader_const::{ColorPreset, InstanceData, Mat4x4};
use kiss3d::resource::PipelineCache;
use wgpu::{BindGroup, BindGroupLayout, Buffer};

/*
 * As defined in the shader, given N components we have
 * - four storage buffers:
 *      + connections : N * MAX_CNX
 *      + operations  : N
 *      + buffer A    : N
 *      + buffer B    : N
 * - two bind group layouts:
 *      + constants : connections, operations
 *      + mutables  : inputs, outputs
 * - three bind groups:
 *      + constants : connections, operations
 *      + A to B
 *      + B to A
 *
 * To optimize sharing data between the CPU and the GPU,
 * we should organize components into three groups of
 * components stored contiguously:
 * - CPU to GPU
 * - GPU to CPU
 * - totally handled by the GPU
 */

/// Define the runner pipeline
pub struct Runner {
    /// GPU Pipeline
    pipeline: PipelineCache,

    /// Data to construct the runner
    data: RunnerData,

    /// flip-flop latch
    flip_flop: bool,
}

/// Data that have been extracted and analyzed from the voxel model to be submitted to the GPU
pub struct RunnerData {
    /// Camera projection matrix
    projection: Mat4x4,

    /// Connections between the components
    connections: Vec<u32>,

    /// Operators of the component (and number of connections)
    operations: Vec<u32>,

    /// Per component data (translation & color index)
    instances_data: Vec<InstanceData>,

    /// Color presets (to be accessed by the color index)
    color_presets: Vec<ColorPreset>,

    /// Initial values for the components (necessary for constant components)
    inits: Vec<u32>,
}

/// Store data for running a logic circuit simulation
pub struct RunnerGpuData {
    /// Buffer allocated in video memory
    buffers: AllocBuffers,

    /// Bind groups for the compute pass
    compute: ComputeBinding,

    /// Bind groups for the render pass
    render: RenderBinding,
}

/// Allocate buffer to store the data
pub struct AllocBuffers {
    /// Projection
    projection: Buffer,

    /// Buffer containing the connections data
    connections: Buffer,

    /// Buffer containing the operations data
    operations: Buffer,

    // Buffer containing the instances data
    instances_data: Buffer,

    /// Buffer containing the color definitions
    color_presets: Buffer,

    /// Swapppable buffers for inputs and outputs
    ping_pong: [Buffer; 2],
}

/// Define the layout for submitting the data to the compute shader
pub struct ComputeLayout {
    /// Layout for connections and operations
    layout_const: BindGroupLayout,

    /// Layout for inputs and outputs
    layout_mut: BindGroupLayout,
}

/// Bind the buffer to the actual shader
pub struct ComputeBinding {
    /// Binding for connections and operations
    bind_const: BindGroup,

    /// Swappable bindings for inputs and outputs
    ping_pong: [BindGroup; 2],
}

/// Define the layout for submitting the data to the render shader
pub struct RenderLayout {
    /// Layout for projection matrix
    layout_proj: BindGroupLayout,

    /// Layout for components and colors
    layout_static: BindGroupLayout,

    /// Layout for process
    layout_process: BindGroupLayout,
}

/// Bind the buffer to the actual shader
pub struct RenderBinding {
    /// Binding for projection matrix
    bind_proj: BindGroup,

    // Binding for components and colors
    bind_static: BindGroup,

    /// Swappable bindings for processed buffer
    ping_pong: [BindGroup; 2],
}
