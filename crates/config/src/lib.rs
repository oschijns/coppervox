/// Specify component types and additional data
pub mod component;

/// Specify how to deserialize colors
pub mod color;

use crate::{color::ColorSet, component::CompDef};
use coppervox_common::Voxel;
use serde::Deserialize;
use std::collections::HashMap;

/// Configuration loaded from a text file such as TOML
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Associate a voxel value with a type of component
    pub map: HashMap<Voxel, CompDef>,

    /// Associate a color index with a color set
    pub colors: Vec<ColorSet>,
}

/// Values transmitted over components are 32-bits
pub type Value = u32;

/// Values transmitted over components are 32-bits
pub const VALUE_SIZE: usize = 32;

impl Config {
    /// Return true if the configuration is valid.
    /// Each color index should map to a valid color set.
    pub fn validate(&self) -> bool {
        let max = self.colors.len();

        // Check each color index
        for def in self.map.values() {
            if def.color_index >= max {
                return false;
            }
        }

        true
    }
}
