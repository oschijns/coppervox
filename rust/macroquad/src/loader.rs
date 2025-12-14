/// Build the runner
pub mod build;

/// Build mesh from quad data
pub mod mesher;

use crate::base::CompType;
use coppervox_analyzer::{Model, Voxel};
use std::collections::HashMap;

/// Configuration loaded from the input files provided
pub struct Config {
    /// Model loaded from the file
    model: Model,

    /// Map voxel values with components type
    map: HashMap<Voxel, CompType>,
}

impl Config {
    /// Create a new config
    #[inline]
    pub fn new(model: Model, map: HashMap<Voxel, CompType>) -> Self {
        Self { model, map }
    }
}
