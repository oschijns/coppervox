use super::pattern::PatternId;
use crate::model::{bounding_box::BoundingBox, VoxBox, VoxDim, VoxIndex};

/// Data associated with a label
pub struct Island<V> {
    /// value of the island
    pub value: V,

    /// bounding box of the island
    pub bounding_box: VoxBox,

    /// the number of voxels in this island
    pub count: usize,

    /// hash of the pattern identified for this island
    pub pattern_id: PatternId,
}

impl<V: Copy> Island<V> {
    /// Generate a new island with the given value,
    /// a null hash pattern and an invalid bounding box.
    pub fn new(value: V) -> Self {
        Self {
            value,
            bounding_box: [
                // set beginning point to max coordinates
                [isize::MAX, isize::MAX, isize::MAX],
                // set ending point to min coordinates
                [isize::MIN, isize::MIN, isize::MIN],
            ],
            count: 0,
            pattern_id: Default::default(),
        }
    }
}

/// Treat islands as bounding boxes
impl<V> BoundingBox for Island<V> {
    /// specify if the given index is inside this box
    #[inline]
    fn contains(&self, index: &VoxIndex) -> bool {
        self.bounding_box.contains(index)
    }

    /// specify if the box is valid
    #[inline]
    fn is_valid(&self) -> bool {
        self.bounding_box.is_valid()
    }

    /// get the size of the box in cells
    #[inline]
    fn size(&self) -> usize {
        self.bounding_box.size()
    }

    /// dimensions of the bounding box
    #[inline]
    fn dimensions(&self) -> VoxDim {
        self.bounding_box.dimensions()
    }

    /// grow the box to include the provided index
    #[inline]
    fn grow_with(&mut self, index: &VoxIndex) {
        self.bounding_box.grow_with(index);
    }
}
