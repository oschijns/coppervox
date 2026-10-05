use coppervox_config::identify::VoxelIdentifier;
use dot_vox::{DotVoxData, Model};

type CVModel = coppervox_common::Model;
type CVVoxel = coppervox_common::Voxel;

/// Number of values to remap
const SIZE: usize = 256;

/// Remap values read from Magica Voxel to processing value
pub struct Remapper<'i> {
    ident: &'i mut VoxelIdentifier,
    map: [CVVoxel; SIZE],
}

/// Create a default Remapper
impl<'i> Remapper<'i> {
    #[inline]
    pub fn new(ident: &'i mut VoxelIdentifier) -> Self {
        let mut map = [0; SIZE];
        for i in 0..SIZE {
            map[i] = i as CVVoxel;
        }
        Self { ident, map }
    }

    /// Convert Magica Voxel data into a Model to be processed
    pub fn convert(&self, magica: &DotVoxData, chunk_dim: [usize; 3]) -> CVModel {
        let mut canvas = CVModel::new(chunk_dim);

        // write the models contained in the file into the canvas
        for model in magica.models.iter() {
            self.write_voxel_model(&mut canvas, model);
        }
        canvas
    }

    fn write_voxel_model(&self, canvas: &mut CVModel, model: &Model) {
        for voxel in model.voxels.iter() {
            let i = [voxel.x as isize, voxel.y as isize, voxel.z as isize];
            // dot_vox stores `voxel.i` as (file palette index - 1), so palette
            // index 1 (the first non-empty color) comes in as 0. Shift back up
            // by 1 so that 0 stays reserved for "no voxel" in our own Model.
            let v = self.map[voxel.i as usize + 1];
            canvas[i] = v;
        }
    }
}
