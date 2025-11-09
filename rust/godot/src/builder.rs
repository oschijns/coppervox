/// Base operations for a builder
mod base;

/// Build buffers to submit to the compute shader
mod buffer;

/// Read quads data to generate a mesh usable by Godot
mod gd_mesher;

use crate::{CompValue, runner::config::CVOperation};
use coppervox_analyzer::{
    Voxel,
    gridmap::hashbrown::{HashMap, HashSet},
};
use godot::{classes::ShaderMaterial, prelude::*};
use std::str::FromStr;

/// Define type use to encode operation type
pub type Operation = i32;

/// CopperVox runner for the logic circuit
#[derive(GodotClass)]
#[class(base=Node, rename=CVBuilder)]
pub struct Builder {
    base: Base<Node>,

    /// Material to assign to the components being generated
    #[export]
    material: Option<Gd<ShaderMaterial>>,

    /// The material should contain a per instance parameter defining the index of the component to pull from the sampler buffer
    #[export]
    instance_index_shader_parameter: GString,

    /// List of instantiated meshes
    instantiated_meshes: Vec<Rid>,
}

/// Given a voxel value, return the operation type for that voxel
#[derive(GodotClass)]
#[class(base=Resource, rename=CVQualifier)]
pub struct Qualifier {
    base: Base<Resource>,

    /// Define mapping between voxel values and operation
    #[export]
    #[var(set = set_mapping)]
    mapping: Array<Gd<Mapping>>,

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
#[class(base=Resource, rename=CVMapping)]
pub struct Mapping {
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

#[godot_api]
impl INode for Builder {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            material: None,
            instance_index_shader_parameter: GString::from_str("instance_index").unwrap(),
            instantiated_meshes: Vec::new(),
        }
    }
}

#[godot_api]
impl IResource for Qualifier {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            mapping: Array::new(),
            evaluated: HashMap::new(),
            actions: HashSet::new(),
        }
    }
}

#[godot_api]
impl IResource for Mapping {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            voxel_type: 0,
            operation: CVOperation::Const,
            voxel_value: 0,
            action: StringName::from_str("").unwrap(),
        }
    }
}

#[godot_api]
impl Builder {}

#[godot_api]
impl Qualifier {
    #[func]
    fn set_mapping(&mut self, mapping: Array<Gd<Mapping>>) {
        self.mapping = mapping;
        for map in self.mapping.iter_shared() {
            let map = map.bind();
            let key = map.voxel_type;

            if map.action.is_empty() {
                if map.operation == CVOperation::Const {
                    // The component is a constant value
                    self.evaluated
                        .insert(key, ComponentType::Constant(map.voxel_value));
                } else {
                    // The component represent an operation
                    self.evaluated
                        .insert(key, ComponentType::Operation(map.operation));
                }
            } else {
                // The component is activated by an action
                let action = map.action.clone();
                self.actions.insert(action.clone());
                self.evaluated.insert(
                    key,
                    ComponentType::Action {
                        value: map.voxel_value,
                        action,
                    },
                );
            }
        }
    }
}
