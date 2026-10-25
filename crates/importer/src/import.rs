use coppervox_common::{Model, color_space::ALPHA_THRESHOLD, rgb::RGB8};
use coppervox_config::identify::VoxelIdentifier;
use std::path::Path;
use ty_math::TyTransformF64;
use voxconv::{DependenciesImpl, ReadFormat, ext::load_with_ext};

/// Read the file at the given path and load voxel data from it
pub fn import_model(
    path: &Path,
    vox_ident: &mut VoxelIdentifier,
    chunk_dim: [usize; 3],
) -> Result<Model, ImportError> {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_ascii_lowercase();

        let format = ident_file_format(&ext)?;

        let vox = load_with_ext(&DependenciesImpl, format, path)?;
        let mut model = Model::new(chunk_dim);
        let mut placements = Vec::new();

        for &root_id in vox.root_hierarchy_node_ids() {
            placements.push((root_id, TyTransformF64::IDENTITY));
        }

        if placements.is_empty() {
            for (_, object) in vox.iter_objects() {
                write_object(
                    &mut model,
                    &vox,
                    object,
                    TyTransformF64::IDENTITY,
                    vox_ident,
                )?;
            }
        } else {
            while let Some((node_id, parent_transform)) = placements.pop() {
                let node = vox
                    .hierarchy_node(node_id)
                    .ok_or(ImportError::InvalidScene)?;
                let transform = parent_transform.compose(&node.transform);

                for &object_id in &node.child_object_ids {
                    let object = vox.object(object_id).ok_or(ImportError::InvalidScene)?;
                    write_object(&mut model, &vox, object, transform, vox_ident)?;
                }

                for &child_id in &node.child_node_ids {
                    placements.push((child_id, transform));
                }
            }
        }

        Ok(model)
    } else {
        Err(ImportError::NoFileExt)
    }
}

fn write_object(
    model: &mut Model,
    vox: &voxconv::ext::VoxconvVoxMain,
    object: &voxcore::VoxObject,
    transform: TyTransformF64,
    vox_ident: &VoxelIdentifier,
) -> Result<(), ImportError> {
    let cell_colors = if vox_ident.color_space.is_some() {
        voxcore::color::resolve_cell_color(vox, object)?
    } else {
        None
    };
    let index_layer = if vox_ident.color_space.is_none() {
        object.iter_layers().last().map(|(layer_id, _)| layer_id)
    } else {
        None
    };

    for voxel_id in object.iter_live() {
        let voxel = if let Some(color_space) = &vox_ident.color_space {
            let Some(cell_colors) = &cell_colors else {
                return Err(ImportError::MissingVoxelColors);
            };
            let [r, g, b, alpha] = cell_colors.color(voxel_id);
            if alpha < ALPHA_THRESHOLD {
                continue;
            }
            color_space.identify_color_index(RGB8::new(r, g, b))?
        } else {
            let Some(layer_id) = index_layer else {
                continue;
            };
            let Some(material_id) = object.voxel_material(voxel_id, layer_id) else {
                continue;
            };
            material_id.to_u32().saturating_add(1)
        };

        let position = object
            .voxel_position(voxel_id)
            .ok_or(ImportError::InvalidScene)?;
        let origin = object.origin();
        let world_position = transform.transform_point(glam::DVec3::new(
            f64::from(position.x) + f64::from(origin.x) + 0.5,
            f64::from(position.y) + f64::from(origin.y) + 0.5,
            f64::from(position.z) + f64::from(origin.z) + 0.5,
        ));
        if !world_position.is_finite() {
            return Err(ImportError::InvalidPosition);
        }
        let coordinate = voxel_coordinate(world_position)?;

        model[[coordinate[0], coordinate[1], coordinate[2]]] = voxel;
    }

    Ok(())
}

fn voxel_coordinate(position: glam::DVec3) -> Result<[isize; 3], ImportError> {
    if !position.is_finite() {
        return Err(ImportError::InvalidPosition);
    }

    let upper_bound = -(isize::MIN as f64);
    let coordinate = [
        (position.x - 0.5).round(),
        (position.y - 0.5).round(),
        (position.z - 0.5).round(),
    ];
    if coordinate
        .iter()
        .any(|value| *value < isize::MIN as f64 || *value >= upper_bound)
    {
        return Err(ImportError::InvalidPosition);
    }

    Ok([
        coordinate[0] as isize,
        coordinate[1] as isize,
        coordinate[2] as isize,
    ])
}

#[rustfmt::skip]
fn ident_file_format(ext: &str) -> Result<ReadFormat, ImportError> {
    match ext {
        "gox"            => Ok(ReadFormat::Goxl),
        "vox"            => Ok(ReadFormat::MVox),
        "qb"             => Ok(ReadFormat::Qb  ),
        "qbt"            => Ok(ReadFormat::Qbt ),
        "qbcl"           => Ok(ReadFormat::Qbcl),
        "vmax"           => Ok(ReadFormat::VMax),
        "voxj" | "voxjz" => Ok(ReadFormat::Voxj),
        _ => Err(ImportError::UnsupportedFile(ext.to_string())),
    }
}

/// Error encountered when importing voxel files
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// No file extension on the file, could not deduce file type
    #[error("Missing extension in the model file")]
    NoFileExt,

    /// Unsupported file type
    #[error("Unsupported file type: \"{0}\"")]
    UnsupportedFile(String),

    /// Invalid or inconsistent scene graph in the voxel file
    #[error("Invalid voxel scene graph")]
    InvalidScene,

    /// The configuration identifies voxels by color, but the file has no base colors
    #[error("The voxel file does not provide colors required by the configuration")]
    MissingVoxelColors,

    /// A scene transform produced a non-finite or out-of-range voxel position
    #[error("Invalid voxel position after applying scene transforms")]
    InvalidPosition,

    #[error("Voxconv: {0}")]
    VoxConv(#[from] voxconv::Error),

    #[error("Voxcore: {0}")]
    VoxCore(#[from] voxcore::Error),

    #[error("Color space: {0}")]
    ColorSpace(#[from] coppervox_common::color_space::LockError),
}

#[cfg(test)]
mod tests {
    use super::{ImportError, voxel_coordinate};

    #[test]
    fn maps_transformed_voxel_centers_to_nearest_grid_cell() {
        assert_eq!(
            voxel_coordinate(glam::DVec3::new(0.5, -0.5, 2.5)).unwrap(),
            [0, -1, 2]
        );
        assert_eq!(
            voxel_coordinate(glam::DVec3::new(1.2, 1.7, 1.1)).unwrap(),
            [1, 1, 1]
        );
    }

    #[test]
    fn rejects_non_finite_positions() {
        assert!(matches!(
            voxel_coordinate(glam::DVec3::new(f64::NAN, 0.5, 0.5)),
            Err(ImportError::InvalidPosition)
        ));
    }
}
