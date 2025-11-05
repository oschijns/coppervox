/// Build buffers to submit to the compute shader
mod buffer;

/// Read quads data to generate a mesh usable by Godot
mod gd_mesher;

use coppervox_analyzer::{Voxel, gridmap::hashbrown::HashMap};
use godot::prelude::*;

/// Define type use to encode operation type
pub type Operation = i32;

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
