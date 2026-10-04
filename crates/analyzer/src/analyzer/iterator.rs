use super::{Label, LabeledModel};
use crate::analyzer::{IslandInfo, ShapeHash, ShapeSet};
use coppervox_common::gridmap::hashbrown::{
    HashMap,
    hash_map::{Iter, Keys, Values},
};

impl LabeledModel {
    /// Iterate over the islands
    pub fn iter(&self) -> Iter<'_, Label, IslandInfo> {
        self.islands.iter()
    }
}

impl<M> ShapeSet<M> {
    /// Iterate over the labels in this set
    #[inline]
    pub fn iter_labels(&self) -> Keys<'_, Label, ShapeHash> {
        self.mapping.keys()
    }

    /// Iterate over the hashes in this set
    #[inline]
    pub fn iter_hashes(&self) -> Keys<'_, ShapeHash, M> {
        self.meshes.keys()
    }

    /// Iterate over the meshes in this set
    #[inline]
    pub fn iter_meshes(&self) -> Values<'_, ShapeHash, M> {
        self.meshes.values()
    }

    /// Iterate over the labels and hashes in this set
    #[inline]
    pub fn iter_labels_hashes(&self) -> Iter<'_, Label, ShapeHash> {
        self.mapping.iter()
    }

    /// Iterate over the hashes and meshes in this set
    #[inline]
    pub fn iter_hashes_meshes(&self) -> Iter<'_, ShapeHash, M> {
        self.meshes.iter()
    }

    /// Iterate over the labels, hashes, and meshes in this set
    #[inline]
    pub fn iter_labels_hashes_meshes(&self) -> IterTuple<'_, M> {
        IterTuple {
            iter: self.mapping.iter(),
            meshes: &self.meshes,
        }
    }
}

/// Iterator over the meshes in a [`ShapeSet`]
pub struct IterTuple<'s, M> {
    /// The underlying iterator over the labels and hashes
    iter: Iter<'s, Label, ShapeHash>,

    /// Reference to map of meshes
    meshes: &'s HashMap<ShapeHash, M>,
}

impl<'s, M> Iterator for IterTuple<'s, M> {
    type Item = (Label, ShapeHash, &'s M);

    /// Next label, hash, and mesh to iterate over
    fn next(&mut self) -> Option<Self::Item> {
        let (label, hash) = self.iter.next()?;
        let mesh = self.meshes.get(hash)?;
        Some((*label, *hash, mesh))
    }
}
