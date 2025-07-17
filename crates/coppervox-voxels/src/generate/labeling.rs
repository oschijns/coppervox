/*!
 * This script use a two-pass implementation of
 * the 6-connected component labeling algorithm.
 * It uses a disjoint-set to regroup connected
 * islands together.
 * */

use super::{
    Cell,
    island::Island,
    pattern::{PatternId, hash_matrix},
};
use crate::{array3, model::VoxModel};
use disjoint_hash_set::DisjointHashSet;
use ndarray::Array3;
use num::traits::ConstZero;
use std::{cmp::min, collections::HashMap};

/// Type used to label islands in the 3D matrices
pub type Label = usize;

/// The result of labeling a voxel model for rendering and logic circuit generation
pub struct LabeledModel<V> {
    /// the voxel model of labeled islands
    pub labeled_model: VoxModel<Label>,

    /// the mapping between labels and islands
    pub islands: HashMap<Label, Island<V>>,

    /// the mapping between pattern identifier and
    /// the corresponding 3D shape to build a mesh from
    pub patterns: HashMap<PatternId, Array3<bool>>,
}

impl<V> LabeledModel<V>
where
    V: Cell + Default + Copy + Eq + ConstZero,
{
    /// Label an input model for rendering and logic circuit generation
    pub fn from_model(model: &VoxModel<V>) -> Self {
        let (labeled_model, mut islands) = connected_component_labeling(model);
        let patterns = identify_patterns(&labeled_model, &mut islands);
        Self {
            labeled_model,
            islands,
            patterns,
        }
    }
}

/// Apply a two pass 6-connected component labeling algorithm
/// to the input model to generate an output a model with labelling
fn connected_component_labeling<V>(
    model: &VoxModel<V>,
) -> (VoxModel<Label>, HashMap<Label, Island<V>>)
where
    V: Cell + Default + Copy + Eq + ConstZero,
{
    // create new labels as we generate islands
    let mut current_label = 1;

    // allocate a matrix for labeling the cells
    let mut labeled_model = VoxModel::<Label>::default();

    // Generate a disjoint set to group labels together.
    // This allows to reassign labels during the second pass.
    let mut disjoint_set = DisjointHashSet::<Label>::new();

    // store the association between temporary labels and initial values
    let mut association = HashMap::<Label, V>::new();

    /* FIRST PASS */
    // iterate over the whole matrix a first time
    for (indexes, &value) in model.0.indexed_iter() {
        if !value.is_empty() {
            // connectivity mask
            // first each bit, indicate if the previous cell
            // on the corresponding axis has the same value
            let mut mask = 0b000usize;

            // get value of previous cells along X, Y and Z
            let [ix, iy, iz] = indexes;

            // Recover the label that was assign to the
            // previous cell along the specified axis.
            // Update the mask accordingly.
            macro_rules! prev_label {
                ($index:expr, $mask:literal) => {{
                    let __index = $index;
                    if value == model[__index] {
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
            // associate the label with the converted value.
            macro_rules! store_label {
                ($label:expr) => {{
                    let __label = $label;
                    labeled_model[indexes] = __label;
                    association.insert(__label, value.into());
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
    let label_count = current_label;
    current_label = 1;

    // reassign the labels based on the groups found within the disjoint set
    let mut remap = HashMap::<Label, Label>::with_capacity(label_count);

    // create a map where labels are associated with data describing an island
    let mut labels = HashMap::<Label, Island<V>>::with_capacity(label_count);

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

        // get any element of the group to find the initial value
        if let Some(label) = group.iter().next() {
            // allocate an island data and store the value associated with the label
            labels.insert(current_label, Island::new(association[label]));
        }

        // create a new label for the next potential group
        current_label += 1;
    }

    // reassign the labels in the matrix
    for cell in labeled_model.0.iter_mut() {
        *cell = remap[cell];
    }

    (labeled_model, labels)
}

/// For each labeled island in the matrix, find the corresponding bounding box.
/// Create a matrix of booleans to identify the pattern corresponding to the label.
fn identify_patterns<V>(
    labeled_model: &VoxModel<Label>,
    islands: &mut HashMap<Label, Island<V>>,
) -> HashMap<PatternId, Array3<bool>> {
    /* FIRST PASS */
    // explore the matrix and identify the boundaries of each island
    for (index, label) in labeled_model.0.indexed_iter() {
        if let Some(island) = islands.get_mut(label) {
            island.grow_with(&index);
        }
    }

    /* SECOND PASS */

    // allocate a map to store unique instances of each pattern
    let mut patterns = HashMap::<PatternId, Array3<bool>>::with_capacity(islands.len());

    // for each island, identify the morphological pattern of the island
    for (label, island) in islands.iter_mut() {
        // dimensions of the matrix generated
        let dimensions = island.dimensions();
        let mut matrix = Array3::<bool>::from_elem(dimensions, false);

        // iterate over the area covered by the island
        for (index, cell_label) in labeled_model.0.bounded_iter(island.bounding_box) {
            if label == cell_label {
                // count the cell as part of the island
                island.count += 1;

                // index to set in the matrix generated
                let matrix_index = array3![for N in {
                    index[N] - island.bounding_box.start[N]
                } -> ndarray::Ix];
                matrix[matrix_index] = true;
            }
        }

        // generate a hash for the matrix
        let hash = hash_matrix(&matrix);
        island.pattern_id = PatternId { dimensions, hash };

        // if the pattern is new, add it to the list
        patterns.entry(island.pattern_id).or_insert(matrix);
    }

    patterns
}
