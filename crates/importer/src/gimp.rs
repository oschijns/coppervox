use coppervox_common::{
    EMPTY_VOXEL, Index, Model,
    color_space::{ALPHA_THRESHOLD, ColorSpace, LockError},
    rgb::RGB8,
};
use xcf::Xcf;

/// Read an Aseprite file and convert its visible frames into voxel slices.
pub fn convert_gimp(
    file: &Xcf,
    color_space: &ColorSpace,
    model: &mut Model,
) -> Result<(), LockError> {
    for (layer_index, layer) in file.layers.iter().enumerate() {
        let width = layer.width;
        let height = layer.height;

        for x in 0..width {
            for y in 0..height {
                if let Some(pixel) = layer.pixel(x, y) {
                    if pixel.a() < ALPHA_THRESHOLD {
                        continue;
                    }
                    let rgb = RGB8::new(pixel.r(), pixel.g(), pixel.b());
                    let voxel = color_space.identify_color_index(rgb)?;
                    if voxel == EMPTY_VOXEL {
                        continue;
                    }

                    let index = [x as Index, layer_index as Index, y as Index];
                    model[index] = voxel;
                }
            }
        }
    }
    Ok(())
}
