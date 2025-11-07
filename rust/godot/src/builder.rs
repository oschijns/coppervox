/// Base operations for a builder
mod base;

/// Build buffers to submit to the compute shader
mod buffer;

/// Read quads data to generate a mesh usable by Godot
mod gd_mesher;

use coppervox_analyzer::{Voxel, gridmap::hashbrown::HashMap};
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
impl Builder {}

/// Given a voxel value, return the operation type for that voxel
#[derive(GodotClass)]
#[class(base=Resource, rename=CVQualifier)]
pub struct Qualifier {
    base: Base<Resource>,

    /// Define mapping between voxel values and operation
    #[export]
    #[var(set = set_mapping)]
    mapping: Dictionary,

    /// Evaluated mapping
    pub(crate) evaluated: HashMap<Voxel, Operation>,
}

#[godot_api]
impl IResource for Qualifier {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            mapping: Dictionary::new(),
            evaluated: HashMap::new(),
        }
    }
}

#[godot_api]
impl Qualifier {
    #[func]
    fn set_mapping(&mut self, mapping: Dictionary) {
        self.mapping = mapping;
        for (key, value) in self.mapping.iter_shared() {
            // The key should be an integer
            let Ok(key) = key.try_to_relaxed::<Voxel>() else {
                continue;
            };

            // The value should be an integer
            let Ok(value) = value.try_to_relaxed::<Operation>() else {
                continue;
            };

            // replicate config in a hashmap for faster lookup
            self.evaluated.insert(key, value);
        }
    }
}
