use coppervox_analyzer::{
    Model, Voxel,
};
use gox::{Block, Data, Gox};

/// Remap values from the Goxel file
pub struct Remapper {

}

/// Create a default Remapper
impl Default for Remapper {
    #[inline]
    fn default() -> Self {
        Self {}
    }
}

impl Remapper {
    /// Convert Magica Voxel data into a Model to be processed
    pub fn convert(&self, goxel: &Gox, chunk_dim: [usize; 3]) -> Model {
        let mut canvas = Model::new(chunk_dim);

        // write the models contained in the file into the canvas
        for model in goxel.data.iter() {
            match model {
                Data::Blocks(block) => {}
                Data::Layers(layers, bounded) => {}
                _ => {}
            }
        }

        canvas
    }
}