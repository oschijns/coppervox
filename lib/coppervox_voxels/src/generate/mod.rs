pub mod connectivity;
pub mod island;
pub mod labeling;
pub mod mesh;
pub mod pattern;

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::{
    array3,
    model::{VoxIndex, VoxModel},
};
use bevy::{
    asset::{Assets, Handle},
    prelude::{Mesh, ResMut},
};
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};
use connectivity::connectivity;
use labeling::label_model;
use mesh::BlockMeshMatrix;
use pattern::PatternId;
use petgraph::{
    csr::{Csr, NodeIndex},
    visit::IntoEdgeReferences,
    Directed,
};

use super::Cell;

/// Define the elements of the circuit
pub struct CircuitData<T> {
    /// The elements of the circuit
    elements: Csr<CircuitElement<T>, (), Directed>,
}

/// Define an element of the circuit
pub struct CircuitElement<T> {
    /// The type of the element
    element_type: T,

    /// Position of the element
    position: VoxIndex,

    /// The mesh used to render the element
    mesh_handle: Handle<Mesh>,
}

impl<T: Copy + Typed> CircuitData<T> {
    /// Analyze the voxel model to prepare a circuit
    /// that can be used to generate bevy entities.
    pub fn generate<C: Cell + Into<T> + Hash, const D: usize>(
        model: &VoxModel<C, D>,
        threshold: usize,
        mut meshes: ResMut<Assets<Mesh>>,
    ) -> Self {
        let labeled = label_model::<C, D, usize, T>(&model);

        // Generate the meshes that will be used by bevy
        let mut mesh_map =
            HashMap::<PatternId, Handle<Mesh>>::with_capacity(labeled.patterns.len());
        for (id, pattern) in labeled.patterns {
            let block_mesh = BlockMeshMatrix::<BoolVoxel>::from_matrix(&pattern);
            mesh_map.insert(
                id,
                meshes.add(block_mesh.build_mesh(
                    [0, 0, 0],
                    array3![for N in {
                        id.dimensions[N]
                    } -> u32],
                )),
            );
        }

        // convert the labeled model into a 3D matrix
        let matrix = labeled.labeled_model.to_matrix();

        // generate the connectivity graph from the matrix
        let labels = HashSet::from_iter(labeled.islands.keys().copied());
        let graph = connectivity(&matrix, &labels);

        // generate the circuit data
        let mut elements = Csr::<CircuitElement<T>, (), Directed>::new();
        let mut node_map = HashMap::<usize, NodeIndex>::with_capacity(labeled.islands.len());
        for (&label, island) in &labeled.islands {
            let index = elements.add_node(CircuitElement {
                element_type: island.value,
                position: island.bounding_box[0],
                mesh_handle: mesh_map[&island.pattern_id].clone(),
            });
            node_map.insert(label, index);
        }

        // iterate over every contacts in the graph
        for (node1, node2, &edge) in graph.edge_references() {
            // connection can only be established between
            // a wire and another component that is not a wire
            let label1 = graph.node_weight(node1);
            let label2 = graph.node_weight(node2);
            if label1 != label2 {
                macro_rules! match_label_with_type {
                    ($label:ident) => {
                        if let Some(island) = labeled.islands.get(&$label) {
                            island.value
                        } else {
                            continue;
                        }
                    };
                }

                let type1 = match_label_with_type![label1];
                let type2 = match_label_with_type![label2];
                if type1.is_wire() != type2.is_wire() {
                    // the two elements should be connected
                    let (the_wire, the_comp) = if type1.is_wire() {
                        (label1, label2)
                    } else {
                        (label2, label1)
                    };

                    // the threshold specify the direction of the connection:
                    // <  threshold: from the wire to the component
                    // >= threshold: from the component to the wire
                    let node_wire = node_map[the_wire];
                    let node_comp = node_map[the_comp];
                    if edge < threshold {
                        elements.add_edge(node_comp, node_wire, ());
                    } else {
                        elements.add_edge(node_wire, node_comp, ());
                    }
                }
            }
        }

        Self { elements }
    }
}

/// Trait for labeling
pub trait Label: Cell + Ord + std::hash::Hash {
    /// Create an initial label for the algorithm
    fn start() -> Self;

    /// Move on to the next usable label
    fn next(&mut self) -> &Self;
}

macro_rules! impl_label {
    ($type:ty) => {
        impl Label for $type {
            fn start() -> Self {
                1
            }

            fn next(&mut self) -> &Self {
                *self += 1;
                self
            }
        }
    };
}

impl_label![u8];
impl_label![u16];
impl_label![u32];
impl_label![u64];
impl_label![usize];
impl_label![i8];
impl_label![i16];
impl_label![i32];
impl_label![i64];
impl_label![isize];

/// Represent a component that has been typed
pub trait Typed {
    /// Specify if the typed component is a wire or not
    fn is_wire(&self) -> bool;
}

#[derive(Default, Clone, Copy, Eq, PartialEq)]
struct BoolVoxel(bool);

impl Voxel for BoolVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if self.0 {
            VoxelVisibility::Opaque
        } else {
            VoxelVisibility::Empty
        }
    }
}

impl MergeVoxel for BoolVoxel {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

impl From<bool> for BoolVoxel {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label() {
        let mut label = u32::start();

        assert_eq![label.is_empty(), false];
        assert_eq![label, 1];
        assert_eq![*label.next(), 2];
    }
}
