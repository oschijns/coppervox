pub mod bounding_box;
pub mod iterator;
pub mod operation;
pub mod pathfinder;
pub mod transform;
pub mod tree;

use crate::array3;
use nalgebra::{vector, Matrix4};
use ndarray::Array3;
use num::integer::{div_floor, mod_floor};
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

/// Re-export cell-wise iterators
pub use iterator::{Iter, IterMut};

/// Index for voxel models
pub type VoxIndex = [isize; 3];

/// Dimensions for voxel models
pub type VoxDim = [usize; 3];

/// Bounding box for voxel models
pub type VoxBox = [VoxIndex; 2];

/// Voxel model data structure:
/// an infinite discrete 3D space of cells
#[derive(Default, Clone)]
pub struct VoxModel<C, const D: usize> {
    // chunks in the infinite matrix
    chunks: HashMap<VoxIndex, Box<Array3<C>>>,

    // empty cell used when indexing a value out of bounds
    out_of_bound: C,
}

/// Indexing for voxel model
impl<C: Default + Copy, const D: usize> Index<VoxIndex> for VoxModel<C, D> {
    type Output = C;

    /// index a value of the infinite matrix using three signed integers
    fn index(&self, index: VoxIndex) -> &Self::Output {
        // find the chunk to look into and the position of the cell in the chunk
        let (chunk_index, cell_index) = split_index::<D>(&index);

        // check if the chunk exists
        if let Some(chunk) = self.chunks.get(&chunk_index) {
            // return a reference to the voxel
            &chunk[cell_index]
        } else {
            // no chunk found return an empty outlier cell
            &self.out_of_bound
        }
    }
}

/// Indexing for voxel model
impl<C: Default + Copy, const D: usize> IndexMut<VoxIndex> for VoxModel<C, D> {
    /// index a value of the infinite matrix using three signed integers
    fn index_mut(&mut self, index: VoxIndex) -> &mut Self::Output {
        // find the chunk to look into and the position of the cell in the chunk
        let (chunk_index, cell_index) = split_index::<D>(&index);

        // recover or insert a new chunk in the map
        let chunk = self
            .chunks
            .entry(chunk_index)
            .or_insert_with(|| Box::new(Array3::<C>::from_elem([D, D, D], C::default())));

        // return a mutable reference to the voxel
        &mut chunk[cell_index]
    }
}

/// Split index into chunk index and cell index
#[inline]
fn split_index<const D: usize>(index: &VoxIndex) -> (VoxIndex, [ndarray::Ix; 3]) {
    (
        array3![for i in { div_floor(index[i], D as isize) } -> isize],
        array3![for i in { mod_floor(index[i], D as isize) } -> ndarray::Ix],
    )
}

/// Transform an index according to a transformation matrix
#[inline]
fn transform_index(transform: &Matrix4<isize>, index: &VoxIndex) -> VoxIndex {
    let t = transform * vector![index[0], index[1], index[2], 1];
    [t.x, t.y, t.z]
}
