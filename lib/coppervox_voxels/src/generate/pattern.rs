use bitvec::{bitvec, order::Msb0};
use fxhash::FxHasher64;
use ndarray::{ArrayBase, Data, Ix3, RawData};
use std::hash::{Hash, Hasher};

use crate::model::VoxDim;

/// Hash value resulting from the morphological analysis of the labeled island
pub type PatternHash = u64;

/// Identify pattern for mesh generation
#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PatternId {
    pub dimensions: VoxDim,
    pub hash: PatternHash,
}

/// for each labeled island in the matrix, compute the connectivity with other matrices
pub fn hash_matrix<R: Data>(matrix: &ArrayBase<R, Ix3>) -> PatternHash
where
    <R as RawData>::Elem: Copy,
    bool: From<<R as RawData>::Elem>,
{
    // allocate a bitbuffer to compute the pattern described by this matrix
    let mut bitbuffer = bitvec![u64, Msb0; 0; matrix.len()];

    // get the dimensions of the matrix
    let (sx, sy, _sz) = matrix.dim();

    // iterate over the matrix
    for ((x, y, z), cell) in matrix.indexed_iter() {
        // check if the cell is filled of empty
        if bool::from(*cell) {
            // index of bit to set in the buffer
            let index = x + sx * (y + sy * z);

            // set the corresponding bit in the buffer
            bitbuffer.set(index, true);
        }
    }

    // create a hasher to generate the pattern hash
    let mut hasher = FxHasher64::default();

    // hash the serialized pattern
    bitbuffer.hash(&mut hasher);

    hasher.finish()
}
