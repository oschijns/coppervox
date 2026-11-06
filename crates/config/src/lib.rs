/// Specify component types and additional data
pub mod component;

/// Specify how to deserialize colors
pub mod color;

/// SYstem which identify a voxel type and recover the component config corresponding to it
pub mod identify;

/// Define mapping between voxel color and component type and color activation gradiants
pub mod manifest;

// re-export crates
pub use coppervox_common as common;

/// Values transmitted over components are 32-bits
pub type Value = u32;

/// Values transmitted over components are 32-bits
pub const VALUE_SIZE: usize = 32;
