use super::{LabeledModel, ShapeData, ShapeHash};
use bitvec::{bitvec, order::Msb0};
use gridmap::hashbrown::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

impl LabeledModel {
    /// For each labeled island in the matrix, find the corresponding bounding box.
    /// Create a matrix of booleans to identify the pattern corresponding to the label.
    pub fn identify_shapes(&mut self) -> HashMap<ShapeHash, ShapeData> {
        /* FIRST PASS */
        // explore the matrix and identify the boundaries of each island
        for (index, label) in self.labeled_model.indexed_iter() {
            if let Some(island) = self.islands.get_mut(label) {
                island.bounding_box.grow_with(&index);
            }
        }

        /* SECOND PASS */
        // allocate a map to store unique instances of each pattern
        let mut shapes = HashMap::<ShapeHash, ShapeData>::with_capacity(self.islands.len());

        // Preallocate structure for identifying shapes to convert into meshes
        let mut bitmatrix = bitvec![u64, Msb0; 0; 0];

        // for each island, identify the morphological pattern of the island
        for (label, island) in self.islands.iter_mut() {
            bitmatrix.clear();

            // dimensions of the matrix generated
            let dimensions = island.bounding_box.dimensions();
            let [dx, dy, dz] = dimensions;

            // create a matrix of bits for storing the overall shape of the island
            bitmatrix.resize(dx * dy * dz, false);

            // iterate over the area covered by the island
            for ([ix, iy, iz], cell_label) in self.labeled_model.bounded_iter(island.bounding_box) {
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
            island.shape_hash = Some(hash);

            // if the shape is new, add it to the list
            shapes.entry(hash).or_insert_with(|| ShapeData {
                bitmatrix: bitmatrix.clone(),
                dimensions,
            });
        }

        // Return the shapes that have been identified
        shapes
    }
}
