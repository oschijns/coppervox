use crate::ImportError;
use coppervox_common::{EMPTY_VOXEL, Model, color_space::ALPHA_THRESHOLD, rgb::RGB8};
use coppervox_config::identify::VoxelIdentifier;
use goxl_voxcore::GoxlExt;
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

        if matches!(ext.as_str(), "ase" | "aseprite") {
            return crate::aseprite::import_aseprite(path, vox_ident, chunk_dim)
                .map_err(ImportError::Aseprite);
        }

        let format = ident_file_format(&ext)?;

        let vox = load_with_ext(&DependenciesImpl, format, path)?;
        let mut model = Model::new(chunk_dim);
        let goxl_ext = vox.ext().downcast_ref::<GoxlExt>();
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
                    glam::DVec3::ZERO,
                    vox_ident,
                )?;
            }
        } else {
            while let Some((node_id, parent_transform)) = placements.pop() {
                let node = vox
                    .hierarchy_node(node_id)
                    .ok_or(ImportError::InvalidScene)?;
                let transform = parent_transform.compose(&node.transform);

                if let Some(layer) = goxl_ext.and_then(|ext| ext.layers.get(&node_id)) {
                    for placement in &layer.placements {
                        let object = vox
                            .object(placement.object_id)
                            .ok_or(ImportError::InvalidScene)?;
                        write_object(
                            &mut model,
                            &vox,
                            object,
                            transform,
                            goxl_placement_offset(placement.position),
                            vox_ident,
                        )?;
                    }
                } else {
                    for &object_id in &node.child_object_ids {
                        let object = vox.object(object_id).ok_or(ImportError::InvalidScene)?;
                        write_object(
                            &mut model,
                            &vox,
                            object,
                            transform,
                            glam::DVec3::ZERO,
                            vox_ident,
                        )?;
                    }
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
    offset: glam::DVec3,
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
            if material_id.to_u32() == EMPTY_VOXEL {
                continue;
            }
            material_id.to_u32().saturating_add(1)
        };

        if voxel == EMPTY_VOXEL {
            continue;
        }

        let position = object
            .voxel_position(voxel_id)
            .ok_or(ImportError::InvalidScene)?;
        let origin = object.origin();
        let world_position = transform.transform_point(glam::DVec3::new(
            f64::from(position.x) + f64::from(origin.x) + 0.5,
            f64::from(position.y) + f64::from(origin.y) + 0.5,
            f64::from(position.z) + f64::from(origin.z) + 0.5,
        )) + offset;
        if !world_position.is_finite() {
            return Err(ImportError::InvalidPosition);
        }
        let coordinate = voxel_coordinate(world_position)?;

        model[[coordinate[0], coordinate[1], coordinate[2]]] = voxel;
    }

    Ok(())
}

fn goxl_placement_offset(position: [i32; 3]) -> glam::DVec3 {
    // Goxel is Z-up; voxcore uses Y-up with Goxel's Y axis reflected into -Z.
    glam::DVec3::new(
        f64::from(position[0]),
        f64::from(position[2]),
        -f64::from(position[1]),
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ImportError;

    #[test]
    fn maps_goxl_z_up_placements_to_voxcore_y_up_axes() {
        assert_eq!(
            goxl_placement_offset([4, 7, -2]),
            glam::DVec3::new(4.0, -2.0, -7.0)
        );
    }

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
