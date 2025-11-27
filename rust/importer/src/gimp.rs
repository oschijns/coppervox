use coppervox_analyzer::Model;
use xcf::Xcf;

/// Convert Magica Voxel data into a Model to be processed
pub fn convert_magica(gimp: &Xcf, chunk_dim: [usize; 3]) -> Model {
    let mut canvas = Model::new(chunk_dim);

    canvas
}
