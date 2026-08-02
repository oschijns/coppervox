use super::{LabeledModel, ShapeData, ShapeHash, ShapeSet};
use crate::Label;
use bitvec::{bitvec, order::Lsb0};
use coppervox_common::gridmap::hashbrown::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

impl<M> ShapeSet<M> {
    /// For each labeled island in the matrix, find the corresponding bounding box.
    /// Create a matrix of booleans to identify the pattern corresponding to the label.
    pub fn identify_shapes(model: &mut LabeledModel) -> Self {
        /* FIRST PASS */
        // explore the matrix and identify the boundaries of each island
        for (index, label) in model.labeled_model.indexed_iter() {
            if let Some(island) = model.islands.get_mut(label) {
                island.bounding_box.grow_with(&index);
            }
        }

        // `grow_with` stores the last voxel index seen along each axis, but
        // `dimensions`/`contains`/`bounded_iter` all treat `end` as exclusive.
        // Without this correction, any island that is only one voxel thick
        // along an axis ends up with a zero-sized dimension on that axis
        // (`start == end`), and elongated islands are undercounted by one.
        for island in model.islands.values_mut() {
            for end in island.bounding_box.end.iter_mut() {
                *end += 1;
            }
        }

        /* SECOND PASS */
        let count = model.islands.len();

        // allocate a map to store unique instances of each pattern
        let mut mapping = HashMap::<Label, ShapeHash>::with_capacity(count);
        let mut shapes = HashMap::<ShapeHash, ShapeData>::with_capacity(count);
        let meshes = HashMap::<ShapeHash, M>::with_capacity(count);

        // Preallocate structure for identifying shapes to convert into meshes
        let mut bitmatrix = bitvec![u64, Lsb0; 0; 0];

        // for each island, identify the morphological pattern of the island
        for (label, island) in model.islands.iter_mut() {
            bitmatrix.clear();

            // dimensions of the matrix generated
            let dimensions = island.bounding_box.dimensions();
            let [dx, dy, dz] = dimensions;

            // create a matrix of bits for storing the overall shape of the island
            bitmatrix.resize(dx * dy * dz, false);

            // iterate over the area covered by the island
            for ([ix, iy, iz], cell_label) in model.labeled_model.bounded_iter(island.bounding_box)
            {
                if label == cell_label {
                    // count the cell as part of the island
                    island.voxels_count += 1;

                    // index to set in the matrix generated
                    let [sx, sy, sz] = island.bounding_box.start;
                    let x = (ix - sx) as usize;
                    let y = (iy - sy) as usize;
                    let z = (iz - sz) as usize;
                    let index = x + dx * (y + dy * z);
                    bitmatrix.set(index, true);
                }
            }

            // generate a hash for the matrix
            let hash = {
                let mut hasher = DefaultHasher::new();
                bitmatrix.hash(&mut hasher);
                hasher.finish()
            };

            // Assign the hash to the island
            let _ = mapping.insert(*label, hash);

            // if the shape is new, add it to the list
            shapes.entry(hash).or_insert_with(|| ShapeData {
                bitmatrix: bitmatrix.clone(),
                dimensions,
            });
        }

        // Return the shapes that have been identified
        Self {
            mapping,
            shapes,
            meshes,
        }
    }
}
