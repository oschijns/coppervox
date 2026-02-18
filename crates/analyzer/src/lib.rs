//!
//!
//!

/// Analyze the voxel data to deduce components and connections between them
mod analyzer;

use gridmap::GridMap;

/// Type used to represent a voxel
pub type Voxel = u32;

/// Type used to define a model to parse
pub type Model = GridMap<Voxel, 3>;

/// Type used to index the GridMap
pub use gridmap::Index;

// re-export crates
pub use block_mesh;
pub use coppervox_common::gridmap;
pub use petgraph;

// re-export types
pub use analyzer::meshing::Mesher;
pub use analyzer::{Analysis, Connectivity, Error, Info, IslandInfo, Label, QuadsReader, ShapeSet};
