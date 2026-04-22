use crate::{
    loader::processor::{CNX_COUNT, IoOrdering, MeshOrdering},
    shader_const::ColorPreset,
};
use coppervox_config::color::ColorSet;
use kiss3d::{
    color::{Color, Rgba},
    glamx::Vec3,
    prelude::{GpuMesh3d, Material3d, Texture},
    scene::{InstanceData3d, Object3d},
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

/// Store meshes and objects to be rendered by the GPU
pub struct Rendering {
    /// Objects to be rendered as multiple instances sharing the same mesh
    objects: Vec<Object3d>,

    /// Color sets usable on the GPU
    color_presets: Vec<ColorPreset>,
}

/// Material used for rendering
pub type Material = Rc<RefCell<Box<dyn Material3d + 'static>>>;

impl Rendering {
    #[inline]
    pub fn objects(&self) -> &Vec<Object3d> {
        &self.objects
    }

    #[inline]
    pub fn color_presets(&self) -> &Vec<ColorPreset> {
        &self.color_presets
    }

    /// Convert the ordered data into `Object3d`
    pub fn make(
        order: MeshOrdering,
        colors: &[ColorSet],
        texture: Arc<Texture>,
        material: Material,
    ) -> Self {
        // Allocate buffers
        let count = order.components().len();
        let mut objects = Vec::with_capacity(count);

        // Create an instance for every components sharing the same mesh
        let mut instances = Vec::with_capacity(count);

        // Create an array for storing indexes of components and colors
        let mut indexes = Vec::with_capacity(count);

        // Convert the meshes
        for (mesh_index1, mesh) in order.meshes().iter().enumerate() {
            // Identify all components using this mesh
            instances.clear();
            indexes.clear();
            for (comp_index, (_, mesh_index2, color_index, bbox)) in
                order.components().iter().enumerate()
            {
                if mesh_index1 == *mesh_index2 {
                    // Create an instance data for this component
                    let mut inst = InstanceData3d::default();
                    let [x, y, z] = bbox.start;
                    inst.position = Vec3::new(x as f32, y as f32, z as f32);
                    instances.push(inst);

                    // Store indexes of the component (where to read the value from)
                    // and the index of the color set to use to color it
                    indexes.push((comp_index, *color_index));
                }
            }

            // No components use this mesh, skip it
            if instances.is_empty() {
                continue;
            }

            // TODO: avoid cloning the render mesh and convert it in place
            // Convert the mesh into a GPU mesh
            let mesh = Rc::new(RefCell::new(GpuMesh3d::from_render_mesh(
                mesh.as_ref().clone(),
                false,
            )));

            // Default color for the object (white)
            const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);

            // Instantiate a Kiss3D object to render
            let mut object = Object3d::new(mesh, WHITE, texture.clone(), material.clone());
            object.set_instances(&instances);

            // TODO: figure out how to provide extra per instance data
            object.set_user_data(Box::new(indexes.clone()));

            // Store the generated object in the list to render
            objects.push(object);
        }

        // Convert color sets to color presets for the shader
        let mut color_presets = Vec::with_capacity(colors.len());
        for color in colors {
            color_presets.push(ColorPreset {
                off: to_gpu_color(color.off),
                low: to_gpu_color(color.on_low),
                high: to_gpu_color(color.on_high),
            });
        }

        Self {
            objects,
            color_presets,
        }
    }
}

/// Converts an RGBA color to a GPU-compatible color array.
#[inline]
fn to_gpu_color(color: Rgba<u8>) -> [f32; 4] {
    let c = Rgba::<f32>::from(color);
    [c.r, c.g, c.b, c.a]
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
