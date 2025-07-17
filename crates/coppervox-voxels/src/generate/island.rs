use super::pattern::PatternId;
use crate::model::VoxBox;
use gridmap::gridmap::bounding_box::BoundingBox;

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
            bounding_box: BoundingBox {
                // set beginning point to max coordinates
                start: [isize::MAX, isize::MAX, isize::MAX],
                // set ending point to min coordinates
                end: [isize::MIN, isize::MIN, isize::MIN],
            },
            count: 0,
            pattern_id: Default::default(),
        }
    }
}
