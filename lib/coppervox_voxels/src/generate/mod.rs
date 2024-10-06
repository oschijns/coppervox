pub mod connectivity;
pub mod island;
pub mod labeling;
pub mod mesh;
pub mod pattern;

use std::collections::HashSet;

use crate::{array3, model::VoxModel};
use bevy::{
    asset::{Assets, Handle},
    prelude::{Mesh, ResMut},
    utils::HashMap,
};
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};
use connectivity::connectivity;
use labeling::label_model;
use mesh::BlockMeshMatrix;
use pattern::PatternId;
use petgraph::{
    matrix_graph::{MatrixGraph, NodeIndex},
    Directed,
};

use super::Cell;

/// Define the elements of the circuit
pub struct CircuitData<T> {
    /// The elements of the circuit
    elements: MatrixGraph<CircuitElement<T>, (), Directed>,
}

/// Define an element of the circuit
pub struct CircuitElement<T> {
    /// The type of the element
    element_type: T,

    /// Position of the element
    position: [i32; 3],

    /// The mesh used to render the element
    mesh_handle: Handle<Mesh>,
}

impl<T: Copy> CircuitData<T> {
    /// Analyze the voxel model to prepare a circuit
    /// that can be used to generate bevy entities.
    pub fn generate<C: Cell + Into<T>, const D: usize>(
        model: &VoxModel<C, D>,
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
        let mut elements =
            MatrixGraph::<CircuitElement<T>, (), Directed>::with_capacity(labels.len());

        Self { elements }
    }
}

/// Convert a VoxModel into a set of meshes
/*
pub fn generate_meshes<C: Cell + Into<V>, const D: usize, V: Copy>(
    model: &VoxModel<C, D>,
    mut meshes: ResMut<Assets<Mesh>>
) {
    let labeled = label_model::<C, D, usize, V>(&model);

    // Generate the meshes that will be used by bevy
    let mut mesh_map = HashMap::<PatternId, Handle<Mesh>>::with_capacity(labeled.patterns.len());
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
}
*/

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
