/// Allocate the buffers for the compute and the render passes
mod alloc_buffers;

/// Generate the storage buffers and appropriate binding groups for compute pass
mod bind_compute;

/// Generate the storage buffers and appropriate binding groups for render pass
mod bind_render;

/// Execute the runner
mod exec;

use crate::shader_const::{GpuColorGradient, GpuInstanceData, GpuMat4x4};
use coppervox_common::List;
use kiss3d::{event::Key, resource::PipelineCache};
use std::{
    cell::RefCell,
    collections::HashSet,
    num::NonZeroU32,
    rc::Rc,
    time::{Duration, Instant},
};
use wgpu::{BindGroup, BindGroupLayout, Buffer, ComputePipeline};

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
    /// Data to construct the runner
    data: RunnerData,

    /// Component indices and key/value pairs controlled by the keyboard
    input_bindings: List<InputBinding>,

    /// Keys currently held down
    pressed_keys: KeyboardState,

    /// Index of the buffer containing the latest simulation values
    current_buffer: usize,

    /// Interval between simulation ticks
    tick_interval: Duration,

    /// Elapsed time accumulated toward the next simulation tick
    tick_accumulator: Duration,

    /// Time of the previous simulation update
    last_tick: Instant,
}

/// Shared set of keys currently held down by the window.
pub type KeyboardState = Rc<RefCell<HashSet<Key>>>;

/// A keyboard-controlled circuit input.
#[derive(Debug, Clone, Copy)]
pub struct InputBinding {
    /// Component index in the processing buffers
    pub component: usize,

    /// Key that controls this component
    pub key: Key,

    /// Value produced while the key is held
    pub value: u32,
}

impl Runner {
    /// Create a runner from pre-computed circuit data
    pub fn new(data: RunnerData) -> Self {
        Self::with_inputs(
            data,
            NonZeroU32::new(60).expect("60 is non-zero"),
            Box::new([]),
            KeyboardState::default(),
        )
    }

    /// Create a runner with keyboard inputs and a configured simulation rate.
    pub fn with_inputs(
        data: RunnerData,
        tick_rate: NonZeroU32,
        input_bindings: List<InputBinding>,
        pressed_keys: KeyboardState,
    ) -> Self {
        let tick_interval =
            Duration::from_secs_f64(1.0 / f64::from(tick_rate.get())).max(Duration::from_nanos(1));
        Self {
            data,
            input_bindings,
            pressed_keys,
            current_buffer: 0,
            tick_interval,
            tick_accumulator: tick_interval,
            last_tick: Instant::now(),
        }
    }
}

/// Data that have been extracted and analyzed from the voxel model to be submitted to the GPU
pub struct RunnerData {
    /// Camera projection matrix
    projection: GpuMat4x4,

    /// Connections between the components
    connections: List<u32>,

    /// Operators of the component (and number of connections)
    operations: List<u32>,

    /// Per component data (translation & color index)
    instances_data: List<GpuInstanceData>,

    /// Color presets (to be accessed by the color index)
    color_presets: List<GpuColorGradient>,

    /// Initial values for the components (necessary for constant components)
    inits: List<u32>,
}

impl RunnerData {
    /// Package the data extracted from a circuit for submission to the GPU
    pub fn new(
        projection: GpuMat4x4,
        connections: List<u32>,
        operations: List<u32>,
        instances_data: List<GpuInstanceData>,
        color_presets: List<GpuColorGradient>,
        inits: List<u32>,
    ) -> Self {
        Self {
            projection,
            connections,
            operations,
            instances_data,
            color_presets,
            inits,
        }
    }
}

/// Store data for running a logic circuit simulation
pub struct RunnerGpuData {
    /// Buffer allocated in video memory
    buffers: AllocBuffers,

    /// Bind groups for the compute pass
    compute: ComputeBinding,

    /// Pipeline used to run one simulation tick
    compute_pipeline: ComputePipeline,

    /// Bind groups for the render pass
    render: RenderBinding,

    /// Pipeline used to draw the circuit, specialized lazily per MSAA sample count
    render_pipeline: PipelineCache,
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
