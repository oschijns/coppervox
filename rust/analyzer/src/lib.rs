//!
//!
//!

/// Analyze the voxel data to deduce components and connections between them
pub mod analyzer;

/// Import various file format to generate a model made of voxels
pub mod importer;

use gridmap::GridMap;

/// Type used to represent a voxel
pub type Voxel = u32;

/// Type used to define a model to parse
pub type Model = GridMap<Voxel, 3>;

// re-export crates
pub use block_mesh;
pub use gridmap;
