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

/// Define iterators for reading results
mod iterator;

/// Label the initial model into islands
mod labeling;

/// Identify the shapes of the islands in the model
mod shaping;

/// Convert the shapes into meshes
pub mod meshing;

/// Identify connections between the islands
mod connecting;

/// Iterator for reading results
//pub mod iterator;
use bitvec::{order::Lsb0, vec::BitVec};
use block_mesh::{QuadBuffer, QuadCoordinateConfig};
use coppervox_common::{
    Voxel,
    gridmap::{GridMap, gridmap::bounding_box::BoundingBox, hashbrown::HashMap},
};
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
#[derive(Clone, Copy)]
pub struct IslandInfo {
    /// Initial value of the voxels in the island
    voxel_value: Voxel,

    /// The number of voxels in this island
    voxels_count: usize,

    /// The bounding box around this island
    bounding_box: BoundingBox<3>,
}

/// Set of shapes identified to be converted into meshes
pub struct ShapeSet<M> {
    /// Map an island label to the corresponding shape hash
    mapping: HashMap<Label, ShapeHash>,

    /// Store shape data so that it can be converted into meshes
    shapes: HashMap<ShapeHash, ShapeData>,

    /// Meshes generated for displaying the components
    meshes: HashMap<ShapeHash, M>,
}

/// Matrix used by the block_mesh crate to generate the meshes
pub struct ShapeData {
    /// The cells composing this matrix where each cell is encoded over a single bit
    bitmatrix: BitVec<u64, Lsb0>,

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
    /// Mapping between labels and their corresponding node index
    mapping: HashMap<Label, NodeIndex>,

    /// Incidence matrix indicating connections between islands
    graph: DiMatrix<Label, usize>,
}

/// Store data about the voxel model that have been analyzed
pub struct Analysis<M> {
    /// The labeled model
    labeled_model: LabeledModel,

    /// Set of shapes identified
    shape_set: ShapeSet<M>,

    /// Define the connections between the components
    connectivity: Connectivity,
}

/// Information on an island
#[derive(Clone)]
pub struct Info<'m, M> {
    /// The label that was associated with the island
    pub label: Label,

    /// The voxel value of the island
    pub voxel_value: Voxel,

    /// The number of voxels in the island
    pub voxels_count: usize,

    /// The mesh generated for the island
    pub mesh: &'m M,

    /// Position of the island in the world
    pub origin: [isize; 3],

    /// The identifier of the node in the incidence matrix
    pub node_index: NodeIndex,
}

/// Error reporting when accessing information
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("missing island info")]
    NoIsland,

    #[error("missing shape")]
    NoShape,

    #[error("missing mesh")]
    NoMesh,

    #[error("missing connectivity")]
    NotConnected,

    #[error("missing label")]
    NoLabel,
}
