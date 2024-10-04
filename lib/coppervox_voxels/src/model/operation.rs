use std::collections::VecDeque;

use nalgebra::{Matrix4, Vector4};

use super::{bounding_box::is_in_boxes, VoxBox, VoxIndex, VoxModel};
use crate::Cell;

/// Implement some operations for voxel models
impl<C: Cell, const D: usize> VoxModel<C, D> {
    /// Apply the given transformation to generate a new matrix
    pub fn transform(&self, transform: &Matrix4<isize>) -> Self {
        let mut model = Self::default();

        // iterate over the whole model and transform
        // the index to write in the output model
        for ([ix, iy, iz], cell) in self.iter() {
            let trs = transform * Vector4::new(ix, iy, iz, 1);
            model[[trs.x, trs.y, trs.z]] = *cell;
        }

        model
    }

    /// Given a set of boxes, extract the overlapping part from the voxel model
    pub fn extract(&self, boxes: &[VoxBox]) -> Self {
        let mut model = Self::default();

        // for each box in the list,
        // copy the cells contained in it from one model to the other
        for abox in boxes {
            for (index, cell) in self.bounded_iter(*abox) {
                model[index] = *cell;
            }
        }

        model
    }

    /// Given a set of boxes, extract the overlapping part from the voxel model
    /// And apply the given transform to the generated model
    pub fn extract_with_transform(&self, boxes: &[VoxBox], transform: &Matrix4<isize>) -> Self {
        let mut model = Self::default();

        // for each box in the list,
        // copy the cells contained in it from one model to the other
        for abox in boxes {
            for ([ix, iy, iz], cell) in self.bounded_iter(*abox) {
                let trs = transform * Vector4::new(ix, iy, iz, 1);
                model[[trs.x, trs.y, trs.z]] = *cell;
            }
        }

        model
    }

    /// Given a set of boxes, replace the voxels in the model by the given one
    pub fn replace(&mut self, boxes: &[VoxBox], new_value: C) {
        // for each box in the list, replace the cells by the one provided
        for abox in boxes {
            for (_, cell) in self.bounded_iter_mut(*abox) {
                *cell = new_value;
            }
        }
    }

    /// Given a starting point, perform a flood fill in the model
    /// A additional list of boxes must be given to ensure that the flood will
    /// not run endlessly incase the shape is not closed.
    pub fn flood_fill(&mut self, start_index: VoxIndex, new_value: C, boundaries: &[VoxBox]) {
        let target = self[start_index];

        // if the value is the same, don't do anything
        if target == new_value {
            return;
        }

        // store the indexes of the cells to visit
        let mut queue = VecDeque::<VoxIndex>::new();
        queue.push_back(start_index);

        // repeat until all indexes in a given space have been visited
        while !queue.is_empty() {
            if let Some(index) = queue.pop_front() {
                // check if the current value in the model is the target one
                // and if the index is in the provided boundaries
                // (we access it as a copy to avoid generating chunks needlessly)
                if target == self[index] && is_in_boxes(&index, boundaries) {
                    // replace the value in the model and
                    // add the neighbors to the queue
                    self[index] = new_value;

                    let [ix, iy, iz] = index;
                    queue.push_back([ix - 1, iy, iz]);
                    queue.push_back([ix + 1, iy, iz]);
                    queue.push_back([ix, iy - 1, iz]);
                    queue.push_back([ix, iy + 1, iz]);
                    queue.push_back([ix, iy, iz - 1]);
                    queue.push_back([ix, iy, iz + 1]);
                }
            }
        }
    }
}
