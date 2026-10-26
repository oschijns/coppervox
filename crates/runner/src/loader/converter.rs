use crate::{
    loader::{
        processor::{CNX_COUNT, Connect, IoOrdering, MeshOrdering},
        uni_mesher::UniQuadDataReader,
    },
    shader_const::{GpuColorGradient, GpuInstanceData, GpuOp},
};
use coppervox_common::List;
use kiss3d::{procedural::RenderMesh, resource::Material3d};
use std::{cell::RefCell, rc::Rc};

/*
 * TODO:
 * Rendering struct was designed for the instancing approach.
 * It is no longer relevant in the unified approach.
 */

/// Store meshes and objects to be rendered by the GPU
pub struct RenderingData {
    /// GPU mesh to render
    pub render_mesh: RenderMesh,

    /// Instances data usable on the GPU
    pub instances_data: List<GpuInstanceData>,

    /// Color sets usable on the GPU
    pub color_presets: List<GpuColorGradient>,
}

/// Material used for rendering
pub type Material = Rc<RefCell<Box<dyn Material3d + 'static>>>;

impl RenderingData {
    /// GPU mesh to render
    #[inline]
    pub fn render_mesh(&self) -> &RenderMesh {
        &self.render_mesh
    }

    /// Extra data for components
    #[inline]
    pub fn instances_data(&self) -> &[GpuInstanceData] {
        &self.instances_data
    }

    /// Buffer of color presets
    #[inline]
    pub fn color_presets(&self) -> &[GpuColorGradient] {
        &self.color_presets
    }

    /// Convert the ordered data into GPU Mesh
    pub fn make(
        quad_data: &UniQuadDataReader,
        order: &MeshOrdering,
        io_order: &IoOrdering,
        color_presets: List<GpuColorGradient>,
    ) -> Self {
        // Allocate buffers
        let count = order.components().len();

        // Allocate the buffers
        let mut data = Vec::with_capacity(count);
        let mut extra = Vec::<[u32; 2]>::with_capacity(count);

        // Store indices triplet for *Kiss3D* model format
        let mut indices = Vec::<[[u32; 3]; 3]>::new();

        // Convert the meshes
        for (index, (label, mesh, color_index, bbox)) in order.iter_components().enumerate() {
            let io_index = io_order
                .index_of(label)
                .expect("every rendered component has a processing index");
            mesh.push_to(index, io_index, &mut indices, &mut extra);

            // Create instance data for the component
            let [x, y, z] = bbox.start;
            data.push(GpuInstanceData::new(
                [x as f32, y as f32, z as f32],
                color_index as u32,
            ));
        }

        Self {
            render_mesh: quad_data.to_mesh(indices, extra),
            instances_data: data.into_boxed_slice(),
            color_presets,
        }
    }
}

/// Store buffers to be processed on the GPU
pub struct Processing {
    /// Connections between nodes in the processing graph
    pub connections: Vec<u32>,

    /// Operations and number of connections where
    /// the high byte is the type of operation to apply and
    /// the three low bytes are the number of connections.
    pub operations: Vec<u32>,

    /// Initial values for the nodes in the processing graph.
    /// This buffer will initialize both `inputs` and `outputs` in the shader.
    pub initial_values: Vec<u32>,
}

impl Processing {
    #[inline]
    pub fn connections(&self) -> &[u32] {
        &self.connections
    }

    #[inline]
    pub fn operations(&self) -> &[u32] {
        &self.operations
    }

    #[inline]
    pub fn initial_values(&self) -> &[u32] {
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
            let op = (GpuOp::from_type(comp_type) as u32) | (*count as u32);
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
