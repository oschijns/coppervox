pub mod bounding_box;
pub mod iterator;
pub mod operation;
pub mod pathfinder;
pub mod transform;
pub mod tree;

use crate::{array3, tuple3, Cell};
use nalgebra::{vector, Matrix4};
use ndarray::Array3;
use num::integer::{div_floor, mod_floor};
use std::{
    cmp::{max, min},
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

/// Convert the voxel model into a 3D matrix
impl<C: Default + Cell, const D: usize> VoxModel<C, D> {
    /// Convert the voxel model into a 3D matrix
    pub fn to_matrix(&self) -> Array3<C> {
        // find the min and max indexes of the chunks
        let mut min_chunk = [isize::MAX; 3];
        let mut max_chunk = [isize::MIN; 3];
        for index in self.chunks.keys() {
            for i in 0..3 {
                min_chunk[i] = min(min_chunk[i], index[i]);
                max_chunk[i] = max(max_chunk[i], index[i]);
            }
        }

        // find the chunks at the limit of the area
        let mut min_idx = [usize::MAX; 3];
        let mut max_idx = [usize::MIN; 3];
        for (chunk_idx, chunk) in self.chunks.iter() {
            // process to find the min and max index along an axis
            macro_rules! process_axis {
                (min $axis:literal) => {
                    if chunk_idx[$axis] == min_chunk[$axis] {
                        for ((ix, iy, iz), c) in chunk.indexed_iter() {
                            let index = [ix, iy, iz][$axis];
                            if !c.is_empty() && min_idx[$axis] > index {
                                min_idx[$axis] = index;
                            }
                        }
                    }
                };
                (max $axis:literal) => {
                    if chunk_idx[$axis] == max_chunk[$axis] {
                        for ((ix, iy, iz), c) in chunk.indexed_iter() {
                            let index = [ix, iy, iz][$axis];
                            if !c.is_empty() && max_idx[$axis] < index {
                                max_idx[$axis] = index;
                            }
                        }
                    }
                };
            }

            // process all three axis
            process_axis![min 0];
            process_axis![min 1];
            process_axis![min 2];
            process_axis![max 0];
            process_axis![max 1];
            process_axis![max 2];
        }

        // figure out the final dimension of the matrix
        let dim = D as isize;
        let min_point = array3![for N in { min_chunk[N] * dim + min_idx[N] as isize } -> isize];
        let max_point = array3![for N in { max_chunk[N] * dim + max_idx[N] as isize } -> isize];
        let dim = tuple3![for N in { max_point[N] - min_point[N] } -> usize];
        let mut matrix = Array3::<C>::default(dim);

        // fill the matrix with the data contained in the model
        for (index, voxel) in self.iter() {
            let i = array3![for N in { index[N] - min_point[N] } -> usize];
            matrix[i] = *voxel;
        }

        matrix
    }
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
