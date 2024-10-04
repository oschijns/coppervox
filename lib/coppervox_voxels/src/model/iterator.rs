use super::{bounding_box::box_overlaps, VoxBox, VoxIndex, VoxModel};
use crate::{array3, Cell};
use ndarray::{Array3, Ix3};
use std::{collections::hash_map, marker::PhantomData};

/// Default iterator for voxel model
pub type Iter<'a, C, const D: usize> = VoxIter<'a, C, D, FilterDef<C>>;

/// Default mutable iterator for voxel model
pub type IterMut<'a, C, const D: usize> = VoxIterMut<'a, C, D, FilterDef<C>>;

/// Iterator for voxel model bounded by a box
pub type BoundedIter<'a, C, const D: usize> = VoxIter<'a, C, D, FilterBox<C, D>>;

/// Mutable iterator for voxel model bounded by a box
pub type BoundedIterMut<'a, C, const D: usize> = VoxIterMut<'a, C, D, FilterBox<C, D>>;

/// Iterate voxel models
#[derive(Clone)]
pub struct VoxIter<'a, C, const D: usize, F> {
    // iterator over chunks
    iter_chunks: ChunkIter<'a, C>,

    // iterator over cells
    iter_cells: Option<CellIter<'a, C>>,

    // store the coordinates of the current chunk
    cached_coords: VoxIndex,

    /// filter to apply while iterating
    filter: F,
}

/// Iterate voxel models
pub struct VoxIterMut<'a, C, const D: usize, F> {
    /// iterator over chunks
    iter_chunks: ChunkIterMut<'a, C>,

    /// iterator over cells
    iter_cells: Option<CellIterMut<'a, C>>,

    /// store the coordinates of the current chunk
    cached_coords: VoxIndex,

    /// filter to apply while iterating
    filter: F,
}

/// Iterator over a hash map containing chunks
pub type ChunkIter<'a, C> = hash_map::Iter<'a, VoxIndex, Box<Array3<C>>>;

/// Mutable iterator over a hash map containing chunks
pub type ChunkIterMut<'a, C> = hash_map::IterMut<'a, VoxIndex, Box<Array3<C>>>;

/// Iterator with indexes over a chunk containing cells
pub type CellIter<'a, C> = ndarray::iter::IndexedIter<'a, C, Ix3>;

/// Mutable iterator with indexes over a chunk containing cells
pub type CellIterMut<'a, C> = ndarray::iter::IndexedIterMut<'a, C, Ix3>;

/// functions to provide iterators for voxel models
impl<C: Default, const D: usize> VoxModel<C, D> {
    /// iterator for a voxel model
    pub fn iter(&self) -> VoxIter<'_, C, D, FilterDef<C>> {
        VoxIter::<'_, C, D, FilterDef<C>> {
            iter_chunks: self.chunks.iter(),
            iter_cells: None,
            cached_coords: Default::default(),
            filter: Default::default(),
        }
    }

    /// mutable iterator for a voxel model
    pub fn iter_mut(&mut self) -> VoxIterMut<'_, C, D, FilterDef<C>> {
        VoxIterMut::<'_, C, D, FilterDef<C>> {
            iter_chunks: self.chunks.iter_mut(),
            iter_cells: None,
            cached_coords: Default::default(),
            filter: Default::default(),
        }
    }

    /// iterator for a voxel model inside of bounding box
    pub fn bounded_iter(&self, bounding_box: VoxBox) -> VoxIter<'_, C, D, FilterBox<C, D>> {
        VoxIter::<'_, C, D, FilterBox<C, D>> {
            iter_chunks: self.chunks.iter(),
            iter_cells: None,
            cached_coords: Default::default(),
            filter: FilterBox::<C, D>::new(bounding_box),
        }
    }

    /// mutable iterator for a voxel model inside of a bounding box
    pub fn bounded_iter_mut(
        &mut self,
        bounding_box: VoxBox,
    ) -> VoxIterMut<'_, C, D, FilterBox<C, D>> {
        VoxIterMut::<'_, C, D, FilterBox<C, D>> {
            iter_chunks: self.chunks.iter_mut(),
            iter_cells: None,
            cached_coords: Default::default(),
            filter: FilterBox::<C, D>::new(bounding_box),
        }
    }

    /// iterator over chunks
    pub fn chunk_iter(&self) -> ChunkIter<'_, C> {
        self.chunks.iter()
    }

    /// mutable iterator over chunks
    pub fn chunk_iter_mut(&mut self) -> ChunkIterMut<'_, C> {
        self.chunks.iter_mut()
    }
}

/// Implement the iterator trait for iterator over infinite matrices
impl<'a, C: Cell, const D: usize, F: IterFilter<C>> Iterator for VoxIter<'a, C, D, F> {
    type Item = (VoxIndex, &'a C);

    // iterate over all the cells in the infinite matrix
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // iterate over the cells of the chunk
            if let Some(iter) = &mut self.iter_cells {
                if let Some((internal_index, cell)) = iter.next() {
                    // reconstruct the index of the voxel
                    let index = array3![
                        for i in {
                            self.cached_coords[i] + internal_index.i as isize
                        } -> isize
                    ];
                    // iterate over the cell only if it pass the filter
                    if self.filter.check_cell(cell, &index) {
                        // return the index and a reference to the voxel
                        return Some((index, cell));
                    }
                }
            }
            // if the cells iterator is missing or it is over,
            // naturally fall to picking a new chunk

            // iterate over the chunks of the map
            if let Some((chunk_index, chunk)) = self.iter_chunks.next() {
                // store the base coordinates of the chunk
                self.cached_coords = array3![
                    for i in {
                        chunk_index[i] * D as isize
                    } -> isize
                ];

                // iterate over the chunk only if it pass the filter
                if self.filter.check_chunk(&self.cached_coords) {
                    // get an iterator for this chunk
                    self.iter_cells = Some(chunk.indexed_iter());
                }

                // ^^^ loop back to the beginning of the function
            } else {
                // no more chunk, we are done
                return None;
            }
        }
    }
}

/// Implement the iterator trait for iterator over infinite matrices
impl<'a, C: Cell, const D: usize, F: IterFilter<C>> Iterator for VoxIterMut<'a, C, D, F> {
    type Item = (VoxIndex, &'a mut C);

    // iterate over all the cells in the infinite matrix
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // iterate over the cells of the chunk
            if let Some(iter) = &mut self.iter_cells {
                if let Some((internal_index, cell)) = iter.next() {
                    // reconstruct the index of the voxel
                    let index = array3![
                        for i in {
                            self.cached_coords[i] + internal_index.i as isize
                        } -> isize
                    ];
                    // iterate over the cell only if it pass the filter
                    if self.filter.check_cell(cell, &index) {
                        // return the index and a reference to the voxel
                        return Some((index, cell));
                    }
                }
            }
            // if the cells iterator is missing or it is over,
            // naturally fall to picking a new chunk

            // iterate over the chunks of the map
            if let Some((chunk_index, chunk)) = self.iter_chunks.next() {
                // store the base coordinates of the chunk
                self.cached_coords = array3![
                    for i in {
                        chunk_index[i] * D as isize
                    } -> isize
                ];

                // iterate over the chunk only if it pass the filter
                if self.filter.check_chunk(&self.cached_coords) {
                    // get an iterator for this chunk
                    self.iter_cells = Some(chunk.indexed_iter_mut());
                }

                // ^^^ loop back to the beginning of the function
            } else {
                // no more chunk, we are done
                return None;
            }
        }
    }
}

/// Filter trait to select the cells returned while iterating a voxel model
pub trait IterFilter<C> {
    /// Specify if the chunk should be iterated over
    fn check_chunk(&self, cached_coords: &VoxIndex) -> bool;

    /// Specify if the cell should be iterated over
    fn check_cell(&self, cell: &C, index: &VoxIndex) -> bool;
}

/// Default filter to skip empty cells only
#[derive(Default)]
pub struct FilterDef<C: Default> {
    phantom: PhantomData<C>,
}

impl<C: Cell> IterFilter<C> for FilterDef<C> {
    /// iterate over all chunks
    fn check_chunk(&self, _cached_coords: &VoxIndex) -> bool {
        true
    }

    /// iterate over all cells that are not empty
    fn check_cell(&self, cell: &C, _index: &VoxIndex) -> bool {
        !cell.is_empty()
    }
}

/// Filter to iterate only over cells inside of a bounding box
pub struct FilterBox<C, const D: usize> {
    phantom: PhantomData<C>,
    bounding_box: VoxBox,
}

impl<C, const D: usize> FilterBox<C, D> {
    /// generate a new filter with the given bounding box
    pub fn new(bounding_box: VoxBox) -> Self {
        Self {
            phantom: Default::default(),
            bounding_box,
        }
    }
}

impl<C: Cell, const D: usize> IterFilter<C> for FilterBox<C, D> {
    /// iterate over chunks that overlaps with the bounding box
    fn check_chunk(&self, cached_coords: &VoxIndex) -> bool {
        // compute the bounding box of the chunk
        let chunk_box = [
            *cached_coords,
            array3!(for N in { cached_coords[N] + D as isize } -> isize),
        ];
        box_overlaps(&self.bounding_box, &chunk_box)
    }

    /// iterate over all cells that are inside the bounding box and not empty
    fn check_cell(&self, cell: &C, index: &VoxIndex) -> bool {
        !cell.is_empty() && self.bounding_box.contains(index)
    }
}
