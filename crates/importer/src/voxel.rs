use crate::ImportError;
use coppervox_common::{
    EMPTY_VOXEL, Index, Model,
    color_space::{ALPHA_THRESHOLD, ColorSpace},
    rgb::RGB8,
};
use glam::{DVec3, IVec3, UVec3};
use goxl_voxcore::GoxlExt;
use ty_math::TyTransformF64;
use voxconv::ext::VoxconvVoxMain;

/// Translation tolerance in units
const TOLERANCE_TRANSLATION: f64 = 1e-6;

/// Rotation tolerance in radians
const TOLERANCE_ROTATION: f64 = 1e-6;

/// Scale tolerance in units
const TOLERANCE_SCALE: f64 = 1e-6;

/// Read the file at the given path and load voxel data from it
pub fn convert_voxel(
    file: VoxconvVoxMain,
    color_space: &ColorSpace,
    model: &mut Model,
) -> Result<(), ImportError> {
    let goxl_ext = file.ext().downcast_ref::<GoxlExt>();
    let mut placements = Vec::new();

    for &root_id in file.root_hierarchy_node_ids() {
        placements.push((root_id, IntTransform::IDENTITY));
    }

    if placements.is_empty() {
        for (_, object) in file.iter_objects() {
            write_object(
                model,
                &file,
                object,
                IntTransform::IDENTITY,
                IVec3::ZERO,
                color_space,
            )?;
        }
    } else {
        while let Some((node_id, parent_transform)) = placements.pop() {
            let node = file
                .hierarchy_node(node_id)
                .ok_or(ImportError::InvalidScene)?;
            let local_transform = IntTransform::from_transform(node.transform)?;
            let transform = parent_transform.compose(&local_transform)?;

            if let Some(layer) = goxl_ext.and_then(|ext| ext.layers.get(&node_id)) {
                for placement in &layer.placements {
                    let object = file
                        .object(placement.object_id)
                        .ok_or(ImportError::InvalidScene)?;
                    write_object(
                        model,
                        &file,
                        object,
                        transform,
                        goxl_placement_offset(IVec3::from_array(placement.position)),
                        color_space,
                    )?;
                }
            } else {
                for &object_id in &node.child_object_ids {
                    let object = file.object(object_id).ok_or(ImportError::InvalidScene)?;
                    write_object(model, &file, object, transform, IVec3::ZERO, color_space)?;
                }
            }

            for &child_id in &node.child_node_ids {
                placements.push((child_id, transform));
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IntTransform {
    translation: IVec3,
    linear: [IVec3; 3],
}

impl IntTransform {
    const IDENTITY: Self = Self {
        translation: IVec3::ZERO,
        linear: [IVec3::X, IVec3::Y, IVec3::Z],
    };

    fn from_transform(transform: TyTransformF64) -> Result<Self, ImportError> {
        let translation = integer_components(transform.position, TOLERANCE_TRANSLATION)?;
        let scale = integer_components(transform.scale, TOLERANCE_SCALE)?;
        if !transform.rotation.is_finite()
            || (transform.rotation.length_squared() - 1.0).abs() > TOLERANCE_ROTATION
        {
            return Err(ImportError::InvalidTransform);
        }

        let rotation = transform.rotation.normalize();
        let rotation_axes = [
            rotation * DVec3::X,
            rotation * DVec3::Y,
            rotation * DVec3::Z,
        ];
        let [Some(x_axis), Some(y_axis), Some(z_axis)] = [
            integer_axis(rotation_axes[0]),
            integer_axis(rotation_axes[1]),
            integer_axis(rotation_axes[2]),
        ] else {
            return Err(ImportError::InvalidTransform);
        };
        if integer_dot(x_axis, y_axis) != 0
            || integer_dot(x_axis, z_axis) != 0
            || integer_dot(y_axis, z_axis) != 0
        {
            return Err(ImportError::InvalidTransform);
        }

        let mut linear = [IVec3::ZERO; 3];
        for (row, entry) in linear.iter_mut().enumerate() {
            *entry = IVec3::new(
                x_axis[row] * scale.x,
                y_axis[row] * scale.y,
                z_axis[row] * scale.z,
            );
        }

        Ok(Self {
            translation,
            linear,
        })
    }

    fn compose(self, child: &Self) -> Result<Self, ImportError> {
        let translation = IVec3::new(
            self.translation.x + checked_dot(self.linear[0], child.translation)?,
            self.translation.y + checked_dot(self.linear[1], child.translation)?,
            self.translation.z + checked_dot(self.linear[2], child.translation)?,
        );

        #[inline]
        fn column_x(matrix: &[IVec3; 3]) -> IVec3 {
            IVec3::new(matrix[0].x, matrix[1].x, matrix[2].x)
        }

        #[inline]
        fn column_y(matrix: &[IVec3; 3]) -> IVec3 {
            IVec3::new(matrix[0].y, matrix[1].y, matrix[2].y)
        }

        #[inline]
        fn column_z(matrix: &[IVec3; 3]) -> IVec3 {
            IVec3::new(matrix[0].z, matrix[1].z, matrix[2].z)
        }

        let mut linear = [IVec3::ZERO; 3];
        for (out, &left) in linear.iter_mut().zip(self.linear.iter()) {
            *out = IVec3::new(
                checked_dot(left, column_x(&child.linear))?,
                checked_dot(left, column_y(&child.linear))?,
                checked_dot(left, column_z(&child.linear))?,
            );
        }

        Ok(Self {
            translation,
            linear,
        })
    }

    fn voxel_coordinate(
        self,
        position: UVec3,
        origin: IVec3,
        offset: IVec3,
    ) -> Result<[Index; 3], ImportError> {
        let source = IVec3::new(
            (position[0] as i32) + origin[0],
            (position[1] as i32) + origin[1],
            (position[2] as i32) + origin[2],
        );
        let doubled_source = source.map(|component| component * 2 + 1);
        let mut result = [0; 3];
        for (row, coordinate) in result.iter_mut().enumerate() {
            let transformed = checked_dot(self.linear[row], doubled_source)?;
            let doubled_center = self.translation[row]
                .checked_mul(2)
                .and_then(|value| value.checked_sub(1))
                .and_then(|value| value.checked_add(transformed))
                .ok_or(ImportError::InvalidPosition)?;
            let cell = round_half_away_from_zero(doubled_center)
                .and_then(|value| value.checked_add(offset[row]))
                .ok_or(ImportError::InvalidPosition)?;
            *coordinate = isize::try_from(cell).map_err(|_| ImportError::InvalidPosition)?;
        }
        Ok(result)
    }
}

fn integer_components(vector: DVec3, tolerance: f64) -> Result<IVec3, ImportError> {
    if !vector.is_finite() {
        return Err(ImportError::InvalidTransform);
    }

    /// Check the coordinate and throw an error if it is beyond the tolerance level.
    fn check_coord(value: f64, tolerance: f64) -> Result<i32, ImportError> {
        let rounded = value.round();
        if (value - rounded).abs() > tolerance
            || rounded < i32::MIN as f64
            || rounded >= -(i32::MIN as f64)
        {
            Err(ImportError::InvalidTransform)
        } else {
            Ok(rounded as i32)
        }
    }

    let x = check_coord(vector.x, tolerance)?;
    let y = check_coord(vector.y, tolerance)?;
    let z = check_coord(vector.z, tolerance)?;
    Ok(IVec3::new(x, y, z))
}

/// A list of the six principal axes in 3D space.
#[rustfmt::skip]
const AXES: [(DVec3, IVec3); 6] = [
    (DVec3::X    , IVec3::X    ),
    (DVec3::NEG_X, IVec3::NEG_X),
    (DVec3::Y    , IVec3::Y    ),
    (DVec3::NEG_Y, IVec3::NEG_Y),
    (DVec3::Z    , IVec3::Z    ),
    (DVec3::NEG_Z, IVec3::NEG_Z),
];

fn integer_axis(axis: DVec3) -> Option<IVec3> {
    let max_distance = 2.0 * (TOLERANCE_ROTATION / 2.0).sin();

    for (ref_axis, integer_axis) in AXES.iter() {
        if (ref_axis - axis).length() <= max_distance {
            return Some(*integer_axis);
        }
    }
    None
}

fn integer_dot(left: IVec3, right: IVec3) -> i32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn checked_dot(left: IVec3, right: IVec3) -> Result<i32, ImportError> {
    let mut result = 0i32;
    for index in 0..3 {
        result = result
            .checked_add(
                left[index]
                    .checked_mul(right[index])
                    .ok_or(ImportError::InvalidPosition)?,
            )
            .ok_or(ImportError::InvalidPosition)?;
    }
    Ok(result)
}

fn round_half_away_from_zero(value_twice: i32) -> Option<i32> {
    let quotient = value_twice.div_euclid(2);
    if value_twice > 0 && value_twice.rem_euclid(2) != 0 {
        quotient.checked_add(1)
    } else {
        Some(quotient)
    }
}

fn write_object(
    model: &mut Model,
    vox: &VoxconvVoxMain,
    object: &voxcore::VoxObject,
    transform: IntTransform,
    offset: IVec3,
    color_space: &ColorSpace,
) -> Result<(), ImportError> {
    let cell_colors = voxcore::color::resolve_cell_color(vox, object)?;

    for voxel_id in object.iter_live() {
        let voxel = {
            let Some(cell_colors) = &cell_colors else {
                return Err(ImportError::MissingVoxelColors);
            };
            let [r, g, b, alpha] = cell_colors.color(voxel_id);
            if alpha < ALPHA_THRESHOLD {
                continue;
            }
            color_space.identify_color_index(RGB8::new(r, g, b))?
        };

        if voxel == EMPTY_VOXEL {
            continue;
        }

        let position = object
            .voxel_position(voxel_id)
            .ok_or(ImportError::InvalidScene)?;
        let origin = object.origin();
        let coordinate = transform.voxel_coordinate(position, origin, offset)?;

        model[[coordinate[0], coordinate[1], coordinate[2]]] = voxel;
    }

    Ok(())
}

#[inline]
fn goxl_placement_offset(vec: IVec3) -> IVec3 {
    // Goxel is Z-up; voxcore uses Y-up with Goxel's Y axis reflected into -Z.
    IVec3::new(vec.x, vec.z, -vec.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ImportError;

    #[test]
    fn maps_goxl_z_up_placements_to_voxcore_y_up_axes() {
        assert_eq!(
            goxl_placement_offset(IVec3::new(4, 7, -2)),
            IVec3::new(4, -2, -7)
        );
    }

    #[test]
    fn maps_integer_voxel_centers_to_grid_cells() {
        assert_eq!(
            IntTransform::IDENTITY
                .voxel_coordinate(UVec3::ZERO, IVec3::new(0, -1, 2), IVec3::ZERO)
                .unwrap(),
            [0, -1, 2]
        );
        assert_eq!(
            IntTransform::IDENTITY
                .voxel_coordinate(UVec3::new(1, 2, 3), IVec3::ZERO, IVec3::ZERO)
                .unwrap(),
            [1, 2, 3]
        );
    }

    #[test]
    fn snaps_near_integer_voxel_transforms() {
        let transform = TyTransformF64::new(
            glam::DVec3::new(1.0 + 1e-8, -2.0, 3.0),
            glam::DQuat::from_rotation_z(std::f64::consts::FRAC_PI_2 + 1e-8),
            glam::DVec3::new(2.0, 1.0, -1.0),
        );

        let integer_transform = IntTransform::from_transform(transform).unwrap();
        assert_eq!(integer_transform.translation, IVec3::new(1, -2, 3));
        assert_eq!(
            integer_transform.linear,
            [IVec3::NEG_Y, IVec3::X * 2, IVec3::NEG_Z]
        );
        assert_eq!(
            integer_transform
                .voxel_coordinate(UVec3::ZERO, IVec3::ZERO, IVec3::ZERO)
                .unwrap(),
            [0, -2, 2]
        );
    }

    #[test]
    fn composes_integer_transforms_without_float_arithmetic() {
        let parent = IntTransform::from_transform(TyTransformF64::new(
            glam::DVec3::new(1.0, 0.0, 0.0),
            glam::DQuat::from_rotation_z(std::f64::consts::FRAC_PI_2),
            glam::DVec3::ONE,
        ))
        .unwrap();
        let child =
            IntTransform::from_transform(TyTransformF64::from_translation(glam::DVec3::X)).unwrap();

        let world = parent.compose(&child).unwrap();
        assert_eq!(world.translation, IVec3::new(1, 1, 0));
        assert_eq!(world.linear, [IVec3::NEG_Y, IVec3::X, IVec3::Z]);
    }

    #[test]
    fn rejects_non_finite_or_non_integer_grid_transforms() {
        assert!(matches!(
            IntTransform::from_transform(TyTransformF64::from_translation(glam::DVec3::new(
                f64::NAN,
                0.0,
                0.0
            )),),
            Err(ImportError::InvalidTransform)
        ));
        let fractional_translation =
            TyTransformF64::from_translation(glam::DVec3::new(0.0, 0.25, 0.0));
        let fractional_scale = TyTransformF64::new(
            glam::DVec3::ZERO,
            glam::DQuat::IDENTITY,
            glam::DVec3::new(1.5, 1.0, 1.0),
        );
        let arbitrary_rotation = TyTransformF64::new(
            glam::DVec3::ZERO,
            glam::DQuat::from_rotation_z(0.25),
            glam::DVec3::ONE,
        );

        for transform in [fractional_translation, fractional_scale, arbitrary_rotation] {
            assert!(matches!(
                IntTransform::from_transform(transform),
                Err(ImportError::InvalidTransform)
            ));
        }
    }
}
