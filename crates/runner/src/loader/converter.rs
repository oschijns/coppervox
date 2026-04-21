use super::RcGpuMesh;
use crate::{
    loader::processor::{CNX_COUNT, IoOrdering, MeshOrdering},
    shader_const::ColorPreset,
};
use coppervox_config::color::ColorSet;
use kiss3d::scene::Object3d;

/// Store meshes and objects to be rendered by the GPU
pub struct Rendering {
    /// Objects to be rendered as multiple instances sharing the same mesh
    objects: Vec<Object3d>,

    /// Color sets usable on the GPU
    color_presets: Vec<ColorPreset>,

    /// Reference to the meshes to be rendered
    meshes: Vec<RcGpuMesh>,
}

impl Rendering {
    #[inline]
    pub fn objects(&self) -> &Vec<Object3d> {
        &self.objects
    }

    #[inline]
    pub fn color_presets(&self) -> &Vec<ColorPreset> {
        &self.color_presets
    }

    #[inline]
    pub fn meshes(&self) -> &Vec<RcGpuMesh> {
        &self.meshes
    }

    /// Convert the ordered data into `Object3d`
    pub fn make(order: &MeshOrdering, colors: &[ColorSet]) -> Self {
        // Allocate buffers
        let mut meshes = Vec::with_capacity(order.meshes().len());
        let mut objects = Vec::with_capacity(order.components().len());
        let mut color_presets = Vec::with_capacity(colors.len());

        // TODO : populate

        Self {
            meshes,
            objects,
            color_presets,
        }
    }
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
    pub fn make(order: &IoOrdering) -> Self {
        let count = order.count();

        // Allocate buffers
        let mut connections = Vec::with_capacity(count * CNX_COUNT);
        let mut operations = Vec::with_capacity(count);
        let mut initial_values = Vec::with_capacity(count);

        // TODO : populate

        Self {
            connections,
            operations,
            initial_values,
        }
    }
}
