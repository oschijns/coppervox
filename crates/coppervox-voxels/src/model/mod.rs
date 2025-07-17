pub mod operation;
pub mod pathfinder;
pub mod transform;
pub mod tree;

use gridmap::{GridMap, gridmap::bounding_box::BoundingBox};
use ndarray::Array3;
use num::traits::ConstZero;
use std::ops::{Index, IndexMut};

use crate::Cell;

/// Index for voxel models
pub type VoxIndex = [isize; 3];

/// Dimensions for voxel models
pub type VoxDim = [usize; 3];

/// Bounding box for voxel models
pub type VoxBox = BoundingBox<3>;

// TODO : use gridmap crate

/// Voxel model data structure:
/// an infinite discrete 3D space of cells
pub struct VoxModel<C>(pub GridMap<C, 3>);

/// Initialize a new empty model
impl<C> Default for VoxModel<C>
where
    C: ConstZero,
{
    fn default() -> Self {
        Self(GridMap::default())
    }
}

/// Convert the voxel model into a 3D matrix
impl<C: Cell + Default + Copy + ConstZero> VoxModel<C> {
    /// Convert the voxel model into a 3D matrix
    pub fn to_matrix(&self) -> Array3<C> {
        let boundaries = self.0.boundaries();
        let minp = boundaries.start;
        let dim = boundaries.dimensions();
        let mut matrix = Array3::<C>::default(dim);

        // fill the matrix with the data contained in the model
        for (index, voxel) in self.0.indexed_iter() {
            macro_rules! get {
                ($i:literal) => {
                    (index[$i] - minp[$i]) as usize
                };
            }
            let i = [get!(0), get!(1), get!(2)];
            matrix[i] = *voxel;
        }
        matrix
    }
}

/// Indexing for voxel model
impl<C> Index<VoxIndex> for VoxModel<C>
where
    C: Copy + ConstZero,
{
    type Output = C;

    /// index a value of the infinite matrix using three signed integers
    #[inline]
    fn index(&self, index: VoxIndex) -> &Self::Output {
        self.0.index(index)
    }
}

/// Indexing for voxel model
impl<C> IndexMut<VoxIndex> for VoxModel<C>
where
    C: Default + Copy + ConstZero,
{
    /// index a value of the infinite matrix using three signed integers
    #[inline]
    fn index_mut(&mut self, index: VoxIndex) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}
