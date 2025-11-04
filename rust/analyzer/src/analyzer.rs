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

/// Basic operations
mod base;

/// Label the initial model into islands
mod labeling;

/// Identify the shapes of the islands in the model
mod shaping;

/// Convert the shapes into meshes
mod meshing;

/// Identify connections between the islands
mod connecting;

/// Iterator for reading results
pub mod iterator;

use crate::Voxel;
use bitvec::{order::Msb0, vec::BitVec};
use block_mesh::{QuadBuffer, QuadCoordinateConfig};
use gridmap::{GridMap, gridmap::bounding_box::BoundingBox, hashbrown::HashMap};
use petgraph::matrix_graph::{DiMatrix, NodeIndex};

/// Type used for labeling islands
pub type Label = u32;

/// Type used to identify a shape
pub type ShapeHash = u64;

/// Labeled voxel model
pub struct LabeledModel {
    /// The labeled model
    labeled_model: GridMap<Label, 3>,

    /// List of the islands identified in this model
    islands: HashMap<Label, IslandInfo>,
}

/// Define an Island data
struct IslandInfo {
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
pub struct ShapeData {
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
pub struct Connectivity {
    /// Incidence matrix indicating connections between islands
    graph: DiMatrix<Label, usize>,

    /// Mapping between labels and their corresponding node index
    mapping: HashMap<Label, NodeIndex>,
}

/// Store data about the voxel model that have been analyzed
pub struct Analysis<M> {
    /// The labeled model
    labeled_model: LabeledModel,

    /// Meshes generated for displaying the components
    meshes: HashMap<ShapeHash, M>,

    /// Define the connections between the components
    connectivity: Connectivity,
}

/// Information on an island
#[derive(Clone)]
pub struct Info<M> {
    /// The voxel value of the island
    pub voxel_value: Voxel,

    /// The number of voxels in the island
    pub voxels_count: usize,

    /// The mesh generated for the island
    pub mesh: M,

    /// The identifier of the node in the incidence matrix
    pub node_index: NodeIndex,
}

/// Error reporting when accessing information
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("No island exists for the label")]
    NoInfo,

    #[error("No mesh was generated for the label")]
    NoMesh,

    #[error("No connectivity was evaluated for the label")]
    NotConnected,
}
