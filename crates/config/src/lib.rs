/// Specify component types and additional data
pub mod component;

/// Specify how to deserialize colors
pub mod color;

use crate::component::CompDef;
use coppervox_common::Voxel;
use serde::Deserialize;
use std::collections::HashMap;

/// Configuration loaded from a text file such as TOML
#[derive(Debug, Deserialize)]
pub struct Config(pub HashMap<Voxel, CompDef>);

/// Values transmitted over components are 32-bits
pub type Value = u32;

/// Values transmitted over components are 32-bits
pub const VALUE_SIZE: usize = 32;
