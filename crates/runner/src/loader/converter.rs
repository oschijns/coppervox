use crate::{
    loader::processor::{CNX_COUNT, Connect, IoOrdering, MeshOrdering},
    shader_const::{ColorPreset, InstanceData, Mat4x4, Op, Vertex},
};
use coppervox_config::color::ColorSet;
use std::ops::Range;

/// Store meshes and instances to be rendered by the GPU.
///
/// Meshes and instances are laid out so that all instances sharing a mesh
/// are contiguous, allowing the whole scene to be rendered with one
/// `draw_indexed` call per unique mesh shape (see [`draws`](Rendering::draws)),
/// regardless of how many instances of that shape exist.
pub struct Rendering {
    /// Vertices of every unique mesh, concatenated together
    vertices: Vec<Vertex>,

    /// Triangle indices of every unique mesh, concatenated together.
    /// Indices are local to their mesh; `MeshDraw::base_vertex` offsets them.
    indices: Vec<u32>,

    /// Per-instance data, grouped by mesh
    instances: Vec<InstanceData>,

    /// One draw call per unique mesh actually in use
    draws: Vec<MeshDraw>,

    /// Color sets usable on the GPU
    color_presets: Vec<ColorPreset>,
}

/// Parameters for a single `draw_indexed` call rendering every instance of one mesh
#[derive(Clone)]
pub struct MeshDraw {
    /// Range within the global index buffer covering this mesh
    pub indices: Range<u32>,

    /// Offset within the global vertex buffer where this mesh's vertices start
    pub base_vertex: i32,

    /// Range within the global instance buffer covering the instances of this mesh
    pub instances: Range<u32>,
}

impl Rendering {
    #[inline]
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    #[inline]
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    #[inline]
    pub fn instances(&self) -> &[InstanceData] {
        &self.instances
    }

    #[inline]
    pub fn draws(&self) -> &[MeshDraw] {
        &self.draws
    }

    #[inline]
    pub fn color_presets(&self) -> &[ColorPreset] {
        &self.color_presets
    }

    /// Convert the ordered data into flat buffers ready to be uploaded to the GPU
    pub fn make(mesh_order: &MeshOrdering, io_order: &IoOrdering, colors: &[ColorSet]) -> Self {
        let mesh_count = mesh_order.meshes().len();
        let comp_count = mesh_order.components().len();

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut instances = Vec::with_capacity(comp_count);
        let mut draws = Vec::with_capacity(mesh_count);

        for (mesh_index, mesh) in mesh_order.meshes().iter().enumerate() {
            let base_vertex = vertices.len() as i32;
            let index_start = indices.len() as u32;
            let instance_start = instances.len() as u32;

            for &(label, comp_mesh_index, color_index, bbox) in mesh_order.components() {
                if comp_mesh_index != mesh_index {
                    continue;
                }

                // Both orderings are derived from the same set of identified
                // components, so every label found here must also be present
                // in the IO ordering.
                let process_index = io_order
                    .index_of(label)
                    .expect("label present in MeshOrdering must also be present in IoOrdering");

                let [x, y, z] = bbox.start;
                instances.push(InstanceData {
                    transform: translation(x as f32, y as f32, z as f32),
                    colors_index: color_index as u32,
                    process_index: process_index as u32,
                    _padding: [0; 2],
                });
            }

            let instance_count = instances.len() as u32 - instance_start;
            if instance_count == 0 {
                continue;
            }

            vertices.extend_from_slice(&mesh.vertices);
            indices.extend_from_slice(&mesh.indices);

            draws.push(MeshDraw {
                indices: index_start..(indices.len() as u32),
                base_vertex,
                instances: instance_start..(instance_start + instance_count),
            });
        }

        let color_presets = colors
            .iter()
            .map(|set| ColorPreset {
                off: to_gpu_color(set.off.r, set.off.g, set.off.b, set.off.a),
                low: to_gpu_color(set.on_low.r, set.on_low.g, set.on_low.b, set.on_low.a),
                high: to_gpu_color(set.on_high.r, set.on_high.g, set.on_high.b, set.on_high.a),
            })
            .collect();

        Self {
            vertices,
            indices,
            instances,
            draws,
            color_presets,
        }
    }
}

/// Build a column-major translation matrix, matching WGSL's `mat4x4<f32>` layout
#[inline]
fn translation(x: f32, y: f32, z: f32) -> Mat4x4 {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [x, y, z, 1.0],
    ]
}

/// Converts an 8-bits-per-channel color into a GPU-compatible float color
#[inline]
fn to_gpu_color(r: u8, g: u8, b: u8, a: u8) -> [f32; 4] {
    const F: f32 = 1.0 / u8::MAX as f32;
    [r as f32 * F, g as f32 * F, b as f32 * F, a as f32 * F]
}

/// Store buffers to be processed on the GPU
pub struct Processing {
    /// Connections between nodes in the processing graph
    connections: Vec<u32>,

    /// Operations and number of connections where
    /// the high byte is the type of operation to apply and
    /// the three low bytes are the number of connections.
    operations: Vec<u32>,

    /// Initial values for the nodes in the processing graph.
    /// This buffer will initialize both `inputs` and `outputs` in the shader.
    initial_values: Vec<u32>,
}

impl Processing {
    #[inline]
    pub fn connections(&self) -> &Vec<u32> {
        &self.connections
    }

    #[inline]
    pub fn operations(&self) -> &Vec<u32> {
        &self.operations
    }

    #[inline]
    pub fn initial_values(&self) -> &Vec<u32> {
        &self.initial_values
    }

    /// Convert the ordered data into raw buffers
    pub fn make(order: &IoOrdering, connect: &Connect) -> Self {
        let count = order.count();

        // Allocate buffers
        let mut connections = Vec::with_capacity(count * CNX_COUNT);
        let mut operations = Vec::with_capacity(count);
        let mut initial_values = Vec::with_capacity(count);

        // Iterate over both array to generate buffers
        for ((_, comp_type), (count, conn)) in
            order.components().iter().zip(connect.buffer().iter())
        {
            for &c in conn.iter() {
                connections.push(c as u32);
            }

            // Convert component type to operation and combine it with connections count
            let op = (Op::from_type(comp_type) as u32) | (*count as u32);
            operations.push(op);

            // Get the initial value for the component (if it is a constant)
            initial_values.push(comp_type.start_value());
        }

        Self {
            connections,
            operations,
            initial_values,
        }
    }
}
