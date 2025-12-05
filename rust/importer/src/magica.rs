use coppervox_analyzer;
use dot_vox::{DotVoxData, Model, Size};

type CVModel = coppervox_analyzer::Model;
type CVVoxel = coppervox_analyzer::Voxel;

/// Number of values to remap
const SIZE: usize = 256;

/// Remap values read from Magica Voxel to processing value
pub struct Remapper([CVVoxel; SIZE]);

/// Create a default Remapper
impl Default for Remapper {
    #[inline]
    fn default() -> Self {
        Self([0; SIZE])
    }
}

impl Remapper {
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
            let v = self.0[voxel.i as usize];
            canvas[i] = v;
        }
    }
}
