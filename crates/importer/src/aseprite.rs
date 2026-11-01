use aseprite_loader::loader::{AsepriteFile, LayerSelection, LoadImageError, LoadSpriteError};
use coppervox_common::{
    EMPTY_VOXEL, Index, Model,
    color_space::{ALPHA_THRESHOLD, ColorSpace},
    rgb::RGB8,
};
use thiserror::Error;

/// Read an Aseprite file and convert its visible frames into voxel slices.
pub fn convert_aseprite<'a>(
    file: &AsepriteFile<'a>,
    color_space: &ColorSpace,
    model: &mut Model,
) -> Result<(), AsepriteImportError> {
    const PIXEL: usize = 4;
    const LAYER: LayerSelection = LayerSelection::Visible;

    // Allocate a buffer to store pixel data
    let (width, height) = file.size();
    let width = width as usize;
    let height = height as usize;
    let alloc = width * height * PIXEL;
    let mut work_buffer = Vec::with_capacity(alloc);
    work_buffer.resize(alloc, 0u8);

    for (frame_index, _) in file.frames().iter().enumerate() {
        file.render_frame(frame_index, &mut work_buffer, &LAYER)?;

        for y in 0..height {
            for x in 0..width {
                let pixel_index = (x + y * width) * PIXEL;

                // extract the four channels of the pixel
                if work_buffer[pixel_index + 3] < ALPHA_THRESHOLD {
                    continue;
                }
                let rgb = RGB8::new(
                    work_buffer[pixel_index],
                    work_buffer[pixel_index + 1],
                    work_buffer[pixel_index + 2],
                );
                let voxel = color_space.identify_color_index(rgb)?;
                if voxel == EMPTY_VOXEL {
                    continue;
                }
                let index = [x as Index, frame_index as Index, y as Index];
                model[index] = voxel;
            }
        }
        work_buffer.fill(0);
    }
    Ok(())
}

/// Errors encountered while converting an Aseprite sprite into a voxel model.
#[derive(Debug, Error)]
pub enum AsepriteImportError {
    /// The file could not be parsed.
    #[error("Could not load Aseprite file: {0}")]
    Load(#[from] LoadSpriteError),

    /// A cel's image data could not be decoded.
    #[error("Could not decode Aseprite image: {0}")]
    Image(#[from] LoadImageError),

    /// A pixel color could not be mapped to a voxel index.
    #[error("Color space: {0}")]
    ColorSpace(#[from] coppervox_common::color_space::LockError),
}
