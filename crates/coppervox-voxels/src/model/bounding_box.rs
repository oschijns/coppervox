use super::{VoxBox, VoxDim, VoxIndex};
use crate::array3;

/// Trait for bounding boxes
pub trait BoundingBox {
    /// specify if the given index is inside this box
    fn contains(&self, index: &VoxIndex) -> bool;

    /// specify if the box is valid
    fn is_valid(&self) -> bool;

    /// get the size of the box in cells
    fn size(&self) -> usize;

    /// dimensions of the bounding box
    fn dimensions(&self) -> VoxDim;

    /// grow the box to include the provided index
    fn grow_with(&mut self, index: &VoxIndex);
}

/// Implement bounding box methods for VoxBox
impl BoundingBox for VoxBox {
    /// specify if the given index is inside this box
    #[inline]
    fn contains(&self, index: &VoxIndex) -> bool {
        let [x0, y0, z0] = self[0];
        let [x1, y1, z1] = self[1];
        let [ix, iy, iz] = *index;

        x0 <= ix && ix < x1 && y0 <= iy && iy < y1 && z0 <= iz && iz < z1
    }

    /// specify if the box is valid
    #[inline]
    fn is_valid(&self) -> bool {
        let [x0, y0, z0] = self[0];
        let [x1, y1, z1] = self[1];
        x0 < x1 && y0 < y1 && z0 < z1
    }

    /// get the size of the box in cells
    #[inline]
    fn size(&self) -> usize {
        let [x, y, z] = self.dimensions();
        x * y * z
    }

    /// dimensions of the bounding box
    #[inline]
    fn dimensions(&self) -> VoxDim {
        array3![for N in {
            self[1][N] - self[0][N]
        } -> usize]
    }

    /// grow the box to include the provided index
    fn grow_with(&mut self, index: &VoxIndex) {
        let p0 = &mut self[0];
        seq_macro::seq![I in 0..3 {
            p0[I] = std::cmp::min(p0[I], index[I]);
        }];
        let p1 = &mut self[1];
        seq_macro::seq![I in 0..3 {
            p1[I] = std::cmp::min(p1[I], index[I]);
        }];
    }
}

/// return true if the two boxes overlap
pub fn box_overlaps(box_a: &VoxBox, box_b: &VoxBox) -> bool {
    // destructure the two bounding boxes
    let [[ax0, ay0, az0], [ax1, ay1, az1]] = box_a;
    let [[bx0, by0, bz0], [bx1, by1, bz1]] = box_b;

    // check if the boxes overlap or not
    !(ax1 < bx0 || bx1 < ax0 || ay1 < by0 || by1 < ay0 || az1 < bz0 || bz1 < az0)
}

/// return true if the index is part of one of the boxes in the list
pub fn is_in_boxes(index: &VoxIndex, boxes: &[VoxBox]) -> bool {
    for abox in boxes {
        if abox.contains(index) {
            return true;
        }
    }
    false
}
