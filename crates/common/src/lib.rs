//!
//! Define common data types used in the other crates
//!

/// Handle a color space partitioning
pub mod color_space;

use gridmap::GridMap;

/// Type used to represent a voxel
pub type Voxel = u32;

/// Empty voxel value
pub const EMPTY_VOXEL: Voxel = 0;

/// Type used to define a model to parse
pub type Model = GridMap<Voxel, 3>;

/// Immutable list of elements
pub type List<T> = Box<[T]>;

/// Type used to index the GridMap
pub use gridmap::Index;

// re-export crates
pub use gridmap;
pub use rgb;
