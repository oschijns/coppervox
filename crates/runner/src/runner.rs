/// Allocate the buffers for the compute and the render passes
mod alloc_buffers;

/// Generate the storage buffers and appropriate binding groups for compute pass
mod bind_compute;

/// Generate the storage buffers and appropriate binding groups for render pass
mod bind_render;

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

/// Store data for running a logic circuit simulation
pub struct Runner {}

/// Allocate buffer to store the data
pub struct AllocBuffers {
    /// Buffer containing the connections data
    connections: Buffer,

    /// Buffer containing the operations data
    operations: Buffer,

    /// Buffer containing the components rendering data
    components: Buffer,

    /// Buffer containing the color definitions
    colors: Buffer,

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
