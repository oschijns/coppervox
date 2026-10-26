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
        for island in model.islands.values_mut() {
            for end in &mut island.bounding_box.end {
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
            let [sx, sy, sz] = island.bounding_box.start;
            let [ex, ey, ez] = island.bounding_box.end;
            for iz in sz..ez {
                for iy in sy..ey {
                    for ix in sx..ex {
                        if label != &model.labeled_model[[ix, iy, iz]] {
                            continue;
                        }

                        // count the cell as part of the island
                        island.voxels_count += 1;

                        // index to set in the matrix generated
                        let x = (ix - sx) as usize;
                        let y = (iy - sy) as usize;
                        let z = (iz - sz) as usize;
                        let index = x + dx * (y + dy * z);
                        bitmatrix.set(index, true);
                    }
                }
            }

            // generate a hash for the matrix
            let hash = {
                let mut hasher = DefaultHasher::new();
                dimensions.hash(&mut hasher);
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

#[cfg(test)]
mod tests {
    use super::ShapeSet;
    use crate::{
        Mesher, QuadsReader,
        analyzer::{LabeledModel, ShapeData},
    };
    use block_mesh::{QuadBuffer, QuadCoordinateConfig, RIGHT_HANDED_Y_UP_CONFIG};
    use coppervox_common::Model;

    struct QuadCounter;

    impl QuadsReader<usize> for QuadCounter {
        fn read_quads(&mut self, quads: &QuadBuffer, _: &QuadCoordinateConfig) -> usize {
            quads.groups.iter().map(Vec::len).sum()
        }
    }

    #[test]
    fn single_voxel_component_produces_six_quads() {
        let mut model = Model::new([4, 4, 4]);
        model[[1, 2, -1]] = 1;

        let mut labeled = LabeledModel::from_model(&model);
        let mut shapes = ShapeSet::<usize>::identify_shapes(&mut labeled);
        let mut mesher = Mesher::new(RIGHT_HANDED_Y_UP_CONFIG, 0);
        shapes.build_meshes(&mut mesher, &mut QuadCounter);

        assert_eq!(shapes.shapes.values().next().unwrap().dimensions, [1, 1, 1]);
        assert_eq!(shapes.meshes.values().copied().sum::<usize>(), 6);
    }

    #[test]
    fn same_occupancy_with_different_dimensions_remains_distinct() {
        let mut model = Model::new([8, 8, 8]);
        model[[0, 0, 0]] = 1;
        model[[1, 0, 0]] = 1;
        model[[1, 0, 1]] = 1;
        model[[4, 4, 4]] = 2;
        model[[5, 4, 4]] = 2;
        model[[5, 5, 4]] = 2;

        let mut labeled = LabeledModel::from_model(&model);
        let shapes = ShapeSet::<ShapeData>::identify_shapes(&mut labeled);

        assert_eq!(shapes.shapes.len(), 2);
        assert!(
            shapes
                .shapes
                .values()
                .any(|shape| shape.dimensions == [2, 1, 2])
        );
        assert!(
            shapes
                .shapes
                .values()
                .any(|shape| shape.dimensions == [2, 2, 1])
        );
    }
}
