//!
//! Define common data types used in the other crates
//!

use gridmap::GridMap;

/// Type used to represent a voxel
pub type Voxel = u32;

/// Type used to define a model to parse
pub type Model = GridMap<Voxel, 3>;

/// Type used to index the GridMap
pub use gridmap::Index;

// re-export crates
pub use gridmap;
