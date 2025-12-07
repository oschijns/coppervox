/// Base operations for a builder
mod base;

/// Build buffers to submit to the compute shader
mod buffer;

/// Read quads data to generate a mesh usable by Godot
pub mod gd_mesher;

use crate::{
    CompValue,
    converter::ModelConverter,
    runner::{
        Runner,
        config::{CONTACT_THRESHOLD, CVOperation},
    },
};
use buffer::GeneratedBuffers;
use coppervox_analyzer::{
    Voxel,
    gridmap::hashbrown::{HashMap, HashSet},
};
use godot::{classes::ShaderMaterial, prelude::*};

/// Build the rendering environment
#[derive(GodotClass)]
#[class(init, base=Node, rename=CVBuilder)]
pub struct Builder {
    base: Base<Node>,

    /// Rendering configuration
    #[export]
    render_config: Option<Gd<RenderConfig>>,

    /// Qualify voxels
    #[export]
    qualifier: Option<Gd<VoxelQualifier>>,

    /// Convert a model made of voxel into a circuit schematic
    #[export]
    converter: Option<Gd<ModelConverter>>,

    /// List of instantiated meshes
    instantiated_meshes: Vec<Rid>,

    /// Raw buffers generated to submit to the GPU
    buffers: Option<GeneratedBuffers>,
}

/// Build the rendering environment
#[derive(GodotClass)]
#[class(init, base=Resource, rename=CVRenderConfig)]
pub struct RenderConfig {
    base: Base<Resource>,

    /// Material to assign to the components being generated
    #[export]
    material: Option<Gd<ShaderMaterial>>,

    /// The material should contain a per instance parameter defining the index of the component to pull from the sampler buffer
    #[export]
    #[init(val = "instance_index".into())]
    instance_index_shader_parameter: GString,
}

/// Given a voxel value, return the operation type for that voxel
#[derive(GodotClass)]
#[class(init, base=Resource, rename=CVVoxelQualifier)]
pub struct VoxelQualifier {
    base: Base<Resource>,

    /// Define mapping between voxel values and operation
    #[export]
    #[var(set = set_mapping)]
    mapping: Array<Gd<VoxelMapping>>,

    /// Map voxel type to corresponding operation
    evaluated: HashMap<Voxel, ComponentType>,

    /// Keep track of the number of actions used
    actions: HashSet<StringName>,
}

/// Define how the voxel type is interpreted as a component
#[derive(Clone)]
pub enum ComponentType {
    /// The component is a constant value
    Constant(CompValue),

    /// The component performs an operation
    Operation(CVOperation),

    /// The component is activated by an action
    Action {
        /// Value to use when activated
        value: CompValue,

        /// Action to listen to
        action: StringName,
    },
}

/// Define data related to a voxel value
#[derive(GodotClass)]
#[class(init, base=Resource, rename=CVVoxelMapping)]
pub struct VoxelMapping {
    base: Base<Resource>,

    /// Which voxel type this mapping corresponds to
    #[export]
    voxel_type: Voxel,

    /// Which operation this voxel value corresponds to
    #[export]
    operation: CVOperation,

    /// The voxel value to use for constant type or when activated by an action
    #[export]
    voxel_value: CompValue,

    /// Should this voxel type be activated by a specific action?
    #[export]
    action: StringName,
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderError {
    #[error("Missing resource: CVRenderConfig")]
    MissingRenderConfig,

    #[error("Missing resource: CVVoxelQualifier")]
    MissingQualifier,

    #[error("Missing resource: CVModelConverter")]
    MissingConverter,

    #[error("No analysis of the model have been performed yet")]
    NoAnalysis,

    #[error("Too many connections on a component (maximum: {max}, found {found})")]
    TooManyConnections { max: usize, found: usize },

    #[error("Missing material in CVRenderConfig")]
    MissingMaterial,
}

#[godot_api]
impl INode for Builder {
    /// When the node is deleted, remove the generated Mesh instances too
    #[inline]
    fn exit_tree(&mut self) {
        self.delete_meshes();
    }
}

#[godot_api]
impl Builder {
    /// Read data from the converter and instantiate meshes
    #[func]
    fn generate(&mut self) -> bool {
        match self.generate_internals(CONTACT_THRESHOLD) {
            Ok(()) => true,
            Err(err) => {
                godot_error!("Failed to generate circuit: {err}");
                false
            }
        }
    }

    /// Feed processed data into the runner
    #[func]
    fn feed_to_runner(&mut self, mut runner: Gd<Runner>) -> bool {
        if let Some(buffers) = &self.buffers {
            let mut runner = runner.bind_mut();

            // set activators
            runner.init_activator(buffers.activators.clone());

            // set storage buffers
            let count = buffers.inputs().len();
            runner
                .build(count, buffers.connections(), buffers.operations())
                .is_ok()
        } else {
            false
        }
    }
}

#[godot_api]
impl VoxelQualifier {
    #[func]
    fn set_mapping(&mut self, mapping: Array<Gd<VoxelMapping>>) {
        self.mapping = mapping;
        self.update_mapping();
    }
}
