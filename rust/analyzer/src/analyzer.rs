//! Analyze the model made of voxels to deduce a logic circuit to run.
//! The analyzer follows a set of steps to produce an optimized logical
//! representation of the circuit.
//!
//! First, identical voxels are grouped into islands.
//! Then islands are checked for contact points, which are used for building a
//! directed graph. They are also compared based on their morphology and
//! converted into meshes. Once the logical representation and the meshes are
//! built, we can proceed to build the storage buffers for the compute shader
//! and instantiate the meshes in Godot's RenderServer.

/// Label the initial model into islands
mod labeling;

/// Identify the shapes of the islands in the model
mod shaping;

/// Convert the shapes into meshes
mod meshing;

/// Identify connections between the islands
mod connecting;

use crate::Voxel;
use bitvec::{order::Msb0, vec::BitVec};
use block_mesh::{QuadBuffer, QuadCoordinateConfig};
use gridmap::{GridMap, gridmap::bounding_box::BoundingBox, hashbrown::HashMap};
use petgraph::matrix_graph::{DiMatrix, NodeIndex};

/// Type used for labeling islands
type Label = u32;

type ShapeHash = u64;

/// Labeled voxel model
struct LabeledModel {
    /// The labeled model
    labeled_model: GridMap<Label, 3>,

    /// List of the islands identified in this model
    islands: HashMap<Label, IslandInfo>,
}

/// Define an Island data
struct IslandInfo {
    /// Label of the island
    label: Label,

    /// Initial value of the voxels in the island
    voxel_value: Voxel,

    /// The number of voxels in this island
    voxels_count: usize,

    /// The bounding box around this island
    bounding_box: BoundingBox<3>,

    /// Hash of the shape associated with this island
    shape_hash: Option<ShapeHash>,
}

/// Matrix used by the block_mesh crate to generate the meshes
struct ShapeData {
    /// The cells composing this matrix where each cell is encoded over a single bit
    bitmatrix: BitVec<u64, Msb0>,

    /// Dimensions of the matrix
    dimensions: [usize; 3],
}

/// Trait for reading block_mesh generated quads and convert them into the
/// appropriate data structure for your target engine.
pub trait QuadsReader<M> {
    fn read_quads(&mut self, quads: &QuadBuffer, config: &QuadCoordinateConfig) -> M;
}

/// Define connectivity between islands
struct Connectivity {
    /// Incidence matrix indicating connections between islands
    graph: DiMatrix<Label, usize>,

    /// Mapping between labels and their corresponding node index
    mapping: HashMap<Label, NodeIndex>,
}
