use coppervox_common::{EMPTY_VOXEL, Model, color_space::ALPHA_THRESHOLD, rgb::RGB8};
use coppervox_config::identify::VoxelIdentifier;
use std::{collections::HashMap, fs, path::Path};
use thiserror::Error;

use aseprite_loader::{
    binary::chunks::layer::{LayerFlags, LayerType},
    loader::{AsepriteFile, LoadImageError, LoadSpriteError},
};

/// Read an Aseprite file and convert its visible layers into voxel slices.
///
/// The first frame is used. Layers are placed from top to bottom along the
/// positive Z axis, with the topmost visible normal layer at Z=0.
pub fn import_aseprite(
    path: &Path,
    vox_ident: &VoxelIdentifier,
    chunk_dim: [usize; 3],
) -> Result<Model, AsepriteImportError> {
    let bytes = fs::read(path)?;
    let sprite = AsepriteFile::load(&bytes)?;
    let frame = sprite.frames.first().ok_or(AsepriteImportError::NoFrames)?;
    let raw_frame = sprite
        .file
        .frames
        .first()
        .ok_or(AsepriteImportError::NoFrames)?;
    let color_space = vox_ident
        .color_space
        .as_ref()
        .ok_or(AsepriteImportError::ColorMappingRequired)?;
    let (width, height) = sprite.size();
    let width = usize::from(width);
    let height = usize::from(height);

    let cels: HashMap<_, _> = frame
        .cels
        .iter()
        .map(|cel| (cel.layer_index, cel))
        .collect();
    let mut model = Model::new(chunk_dim);
    let mut group_state = Vec::new();
    let mut visible_layers = Vec::new();
    for (index, layer) in sprite.file.layers.iter().enumerate() {
        while group_state
            .last()
            .is_some_and(|(level, _, _)| *level >= layer.child_level)
        {
            group_state.pop();
        }
        let (parent_visible, parent_opacity) = group_state
            .last()
            .map_or((true, u8::MAX), |(_, visible, opacity)| {
                (*visible, *opacity)
            });
        let visible = parent_visible && layer.flags.contains(LayerFlags::VISIBLE);
        let opacity = u32::from(parent_opacity) * u32::from(layer.opacity) / u32::from(u8::MAX);
        match layer.layer_type {
            LayerType::Group => {
                group_state.push((layer.child_level, visible, opacity as u8));
            }
            LayerType::Normal if visible => visible_layers.push((index, opacity as u8)),
            LayerType::Normal | LayerType::Tilemap | LayerType::Unknown(_) => {}
        }
    }

    for (depth, (layer_index, layer_opacity)) in visible_layers.iter().rev().enumerate() {
        let Some(cel) = cels.get(layer_index) else {
            continue;
        };
        let raw_cel = raw_frame
            .cels
            .get(*layer_index)
            .and_then(Option::as_ref)
            .ok_or(AsepriteImportError::InvalidLayerIndex)?;
        let mut pixels = vec![0; usize::from(cel.size.0) * usize::from(cel.size.1) * 4];
        sprite.load_image(cel.image_index, &mut pixels)?;

        for pixel_y in 0..usize::from(cel.size.1) {
            for pixel_x in 0..usize::from(cel.size.0) {
                let sprite_x = i32::from(cel.origin.0) + pixel_x as i32;
                let sprite_y = i32::from(cel.origin.1) + pixel_y as i32;
                if sprite_x < 0
                    || sprite_y < 0
                    || sprite_x >= width as i32
                    || sprite_y >= height as i32
                {
                    continue;
                }

                let pixel_index = (pixel_y * usize::from(cel.size.0) + pixel_x) * 4;
                let alpha = u32::from(pixels[pixel_index + 3])
                    * u32::from(raw_cel.opacity)
                    * u32::from(*layer_opacity)
                    / (u32::from(u8::MAX) * u32::from(u8::MAX));
                if alpha < u32::from(ALPHA_THRESHOLD) {
                    continue;
                }

                let voxel = color_space.identify_color_index(RGB8::new(
                    pixels[pixel_index],
                    pixels[pixel_index + 1],
                    pixels[pixel_index + 2],
                ))?;
                if voxel == EMPTY_VOXEL {
                    continue;
                }

                model[[
                    sprite_x as isize,
                    (height as i32 - 1 - sprite_y) as isize,
                    depth as isize,
                ]] = voxel;
            }
        }
    }

    Ok(model)
}

/// Errors encountered while converting an Aseprite sprite into a voxel model.
#[derive(Debug, Error)]
pub enum AsepriteImportError {
    /// The file could not be read from disk.
    #[error("Could not read Aseprite file: {0}")]
    Io(#[from] std::io::Error),

    /// The file could not be parsed.
    #[error("Could not load Aseprite file: {0}")]
    Load(#[from] LoadSpriteError),

    /// A cel's image data could not be decoded.
    #[error("Could not decode Aseprite image: {0}")]
    Image(#[from] LoadImageError),

    /// The sprite does not contain any frames.
    #[error("The Aseprite sprite contains no frames")]
    NoFrames,

    /// This importer requires a color-based voxel mapping.
    #[error("Aseprite import requires a color-based voxel mapping")]
    ColorMappingRequired,

    /// A cel refers to a layer that is not present in the sprite.
    #[error("Invalid layer index in Aseprite file")]
    InvalidLayerIndex,

    /// A pixel color could not be mapped to a voxel index.
    #[error("Color space: {0}")]
    ColorSpace(#[from] coppervox_common::color_space::LockError),
}
