use serde::Deserialize;

/// Specify where to place the origin of the
/// model when importing external formats.
#[derive(Default, Deserialize)]
pub enum OriginConfig {
    /// Mesh origin is at the left, bottom or front of the model
    Start,

    /// Mesh origin is at the middle of the model
    #[default]
    Middle,

    /// Mesh origin is at the right, top or back of the model
    End,
}

/// Conversion of Magica Voxel
pub mod magica_voxel {
    use super::OriginConfig;
    use crate::{model::VoxModel, Cell};
    use dot_vox::{Model, Size, Voxel};

    /// Conversion of Magica Voxel into ndarray::Array3
    pub fn convert<C: Cell + From<u8>, const D: usize>(
        model: &Model,
        origin_config: &[OriginConfig; 3],
    ) -> VoxModel<C, D> {
        let Size { x, y, z } = model.size;

        // convert the size read into an offset for storing voxels
        macro_rules! set_offset {
            ($axis:ident $index:literal) => {
                match origin_config[$index] {
                    OriginConfig::Start => 0isize,
                    OriginConfig::Middle => $axis as isize / 2isize,
                    OriginConfig::End => $axis as isize,
                }
            };
        }
        let offset_x = set_offset!(x 0);
        let offset_y = set_offset!(y 2);
        let offset_z = set_offset!(z 1);

        // reorder the coordinates as Magica Voxel uses Z as the up coordinate
        let mut voxel_model = VoxModel::<C, D>::default();

        model.voxels.iter().for_each(|&voxel| {
            let Voxel { x, y, z, .. } = voxel;
            let index = [
                x as isize - offset_x,
                z as isize - offset_z,
                y as isize - offset_y,
            ];
            voxel_model[index] = C::from(voxel.i + 1u8);
        });

        voxel_model
    }
}

/// Conversion of 2D image
pub mod image_2d {
    use crate::{model::VoxModel, Cell};
    use image::GenericImageView;

    use super::OriginConfig;

    /// Conversion of Generic 2D image into ndarray::Array2
    pub fn convert<
        C: Cell + From<<I as GenericImageView>::Pixel>,
        const D: usize,
        I: GenericImageView,
    >(
        image: &I,
        origin_config: &[OriginConfig; 2],
    ) -> VoxModel<C, D> {
        let (sx, sy) = image.dimensions();

        // convert the size read into an offset for storing voxels
        macro_rules! set_offset {
            ($axis:ident $index:literal) => {
                match origin_config[$index] {
                    OriginConfig::Start => 0isize,
                    OriginConfig::Middle => $axis as isize / 2isize,
                    OriginConfig::End => $axis as isize,
                }
            };
        }
        let offset_x = set_offset!(sx 0);
        let offset_y = set_offset!(sy 1);

        // load the image in a top-down view
        let mut voxel_model = VoxModel::<C, D>::default();

        // iterate over the image and store the pixels into the matrix
        image.pixels().for_each(|(x, y, pixel)| {
            let index = [x as isize - offset_x, 0isize, y as isize - offset_y];
            voxel_model[index] = C::from(pixel);
        });

        voxel_model
    }
}
