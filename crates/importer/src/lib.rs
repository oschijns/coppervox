/// Method for loading a common voxel file format and convert it into a model
pub mod import;

/// Load Magica Voxel files
//pub mod magica;

// Load Goxel files
//pub mod goxel;

/// Load GIMP files
pub mod gimp;

// re-export crates
pub use coppervox_common as common;
pub use dot_vox;
