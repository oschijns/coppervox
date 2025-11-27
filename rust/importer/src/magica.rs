use coppervox_analyzer;
use dot_vox::{DotVoxData, Model, Size};

type CVModel = coppervox_analyzer::Model;
type CVVoxel = coppervox_analyzer::Voxel;

/// Convert Magica Voxel data into a Model to be processed
pub fn convert_magica(magica: &DotVoxData, chunk_dim: [usize; 3]) -> CVModel {
    let mut canvas = CVModel::new(chunk_dim);

    // write the models contained in the file into the canvas
    for model in magica.models.iter() {
        write_voxel_model(&mut canvas, model);
    }
    canvas
}

fn write_voxel_model(canvas: &mut CVModel, model: &Model) {
    for voxel in model.voxels.iter() {
        let i = [voxel.x as i32, voxel.y as i32, voxel.z as i32];
        canvas[i] = voxel.i as CVVoxel;
    }
}
