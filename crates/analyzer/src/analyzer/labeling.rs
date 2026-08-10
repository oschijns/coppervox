use super::{IslandInfo, Label, LabeledModel};
use coppervox_common::{
    Model, Voxel,
    gridmap::{GridMap, gridmap::bounding_box::BoundingBox, hashbrown::HashMap},
};
use disjoint_hash_set::DisjointHashSet;
use std::cmp::min;

impl LabeledModel {
    /// Apply a two pass 6-connected component labeling algorithm
    /// to the input model to generate an output a model with labelling
    pub fn from_model(model: &Model) -> Self {
        // create new labels as we generate islands
        let mut current_label = 1;

        // allocate a matrix for labeling the cells
        let mut labeled_model = GridMap::<Label, 3>::new(model.get_chunks_dimensions());

        // Generate a disjoint set to group labels together.
        // This allows to reassign labels during the second pass.
        let mut disjoint_set = DisjointHashSet::<Label>::new();

        // store the association between temporary labels and initial voxels
        let mut association = HashMap::<Label, Voxel>::new();

        /* FIRST PASS */
        // iterate over the whole matrix a first time
        for (indexes, &voxel) in model.indexed_iter() {
            if voxel != 0 {
                // connectivity mask
                // first each bit, indicate if the previous cell
                // on the corresponding axis has the same voxel
                let mut mask = 0b000usize;

                // get voxel of previous cells along X, Y and Z
                let [ix, iy, iz] = indexes;

                // Recover the label that was assign to the
                // previous cell along the specified axis.
                // Update the mask accordingly.
                macro_rules! prev_label {
                    ($index:expr, $mask:literal) => {{
                        let __index = $index;
                        if voxel == model[__index] {
                            mask |= $mask;
                            labeled_model[__index]
                        } else {
                            0
                        }
                    }};
                }

                // indexes of the previous cells for each of the axes
                let label_x = prev_label!([ix - 1, iy, iz], 0b001);
                let label_y = prev_label!([ix, iy - 1, iz], 0b010);
                let label_z = prev_label!([ix, iy, iz - 1], 0b100);

                // Set the label in the label matrix and
                // associate the label with the converted voxel.
                macro_rules! store_label {
                    ($label:expr) => {{
                        let __label = $label;
                        labeled_model[indexes] = __label;
                        association.insert(__label, voxel.into());
                    }};
                }

                match mask {
                    // no connection to surrounding cells, start a new island
                    0b000 => {
                        store_label!(current_label);
                        disjoint_set.insert(current_label);
                        current_label += 1;
                    }

                    // connection only along X
                    0b001 => {
                        store_label!(label_x);
                    }

                    // connection only along Y
                    0b010 => {
                        store_label!(label_y);
                    }

                    // connection only along Z
                    0b100 => {
                        store_label!(label_z);
                    }

                    // connection along X and Y
                    0b011 => {
                        store_label!(min(label_x, label_y));
                        disjoint_set.link(label_x, label_y);
                    }

                    // connection along Y and Z
                    0b110 => {
                        store_label!(min(label_y, label_z));
                        disjoint_set.link(label_y, label_z);
                    }

                    // connection along Z and X
                    0b101 => {
                        store_label!(min(label_z, label_x));
                        disjoint_set.link(label_z, label_x);
                    }

                    // connection along all three axes
                    0b111 => {
                        store_label!(min(label_x, min(label_y, label_z)));
                        disjoint_set.link(label_x, label_y);
                        disjoint_set.link(label_y, label_z);
                    }

                    // should never happen
                    _ => {}
                }
            }
        }

        /* SECOND PASS */
        let label_count = current_label as usize;
        current_label = 1;

        // reassign the labels based on the groups found within the disjoint set
        let mut remap = HashMap::<Label, Label>::with_capacity(label_count);

        // create a map where labels are associated with data describing an island
        let mut islands = HashMap::<Label, IslandInfo>::with_capacity(label_count);

        // regroup labels that are connected under a single label
        for group in disjoint_set.sets() {
            // don't reassign a label for groupds that are empty
            // it shouldn't happen though...
            if group.is_empty() {
                continue;
            }

            // for all labels that are related,
            // specify by which new label they should be replaced
            for element in group.iter() {
                remap.insert(*element, current_label);
            }

            // get any element of the group to find the initial voxel
            if let Some(label) = group.iter().next() {
                // allocate an island data and store the voxel associated with the label
                islands.insert(current_label, IslandInfo::new(association[label]));
            }

            // create a new label for the next potential group
            current_label += 1;
        }

        // reassign the labels in the matrix
        for cell in labeled_model.iter_mut() {
            if let Some(&c) = remap.get(cell) {
                *cell = c;
            }
        }

        Self {
            labeled_model,
            islands,
        }
    }
}

impl IslandInfo {
    /// Generate a new island with the given voxel,
    /// a null hash pattern and an invalid bounding box.
    fn new(voxel_value: Voxel) -> Self {
        Self {
            voxel_value,
            bounding_box: BoundingBox {
                // set beginning point to max coordinates
                start: [isize::MAX, isize::MAX, isize::MAX],
                // set ending point to min coordinates
                end: [isize::MIN, isize::MIN, isize::MIN],
            },
            voxels_count: 0,
        }
    }
}
