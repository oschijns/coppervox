use gridmap::transform::Transform;
use nalgebra::{Vector4, matrix};
use num::integer::mod_floor;

#[derive(Debug, Default, Clone, Copy)]
pub struct Matrix(pub nalgebra::Matrix4<isize>);

impl Matrix {
    /// Build a transformation matrix describing a translation
    pub fn translation(vec: [isize; 3]) -> Self {
        Self(matrix![
             1, 0, 0, vec[0];
             0, 1, 0, vec[1];
             0, 0, 1, vec[2];
             0, 0, 0, 1;
        ])
    }

    /// Build a transformation matrix describing a rotation around the X-axis
    pub fn rotation_x(int_angle: isize) -> Self {
        let (s, c) = int_sin_cos(int_angle);
        Self(matrix![
             1, 0, 0, 0;
             0, c,-s, 0;
             0, s, c, 0;
             0, 0, 0, 1;
        ])
    }

    /// Build a transformation matrix describing a rotation around the Y-axis
    pub fn rotation_y(int_angle: isize) -> Self {
        let (s, c) = int_sin_cos(int_angle);
        Self(matrix![
             c, 0, s, 0;
             0, 1, 0, 0;
            -s, 0, c, 0;
             0, 0, 0, 1;
        ])
    }

    /// Build a transformation matrix describing a rotation around the Z-axis
    pub fn rotation_z(int_angle: isize) -> Self {
        let (s, c) = int_sin_cos(int_angle);
        Self(matrix![
             c,-s, 0, 0;
             s, c, 0, 0;
             0, 0, 1, 0;
             0, 0, 0, 1;
        ])
    }

    /// Build a transformation matrix describing a reflection along the X-axis
    pub fn reflect_x() -> Self {
        Self(matrix![
            -1, 0, 0, 0;
             0, 1, 0, 0;
             0, 0, 1, 0;
             0, 0, 0, 1;
        ])
    }

    /// Build a transformation matrix describing a reflection along the Y-axis
    pub fn reflect_y() -> Self {
        Self(matrix![
             1, 0, 0, 0;
             0,-1, 0, 0;
             0, 0, 1, 0;
             0, 0, 0, 1;
        ])
    }

    /// Build a transformation matrix describing a reflection along the Z-axis
    pub fn reflect_z() -> Self {
        Self(matrix![
             1, 0, 0, 0;
             0, 1, 0, 0;
             0, 0,-1, 0;
             0, 0, 0, 1;
        ])
    }
}

/// Return sine and cosine for angle encoded as π/2 × int_angle
#[inline]
fn int_sin_cos(int_angle: isize) -> (isize, isize) {
    match mod_floor(int_angle, 4) {
        1 => (1, 0),
        2 => (0, -1),
        3 => (-1, 0),
        _ => (0, 1),
    }
}

impl Transform<3> for Matrix {
    fn apply(&self, index: &mut [isize; 3]) {
        let i = self.0 * Vector4::new(index[0], index[1], index[2], 1);
        index[0] = i.x;
        index[1] = i.y;
        index[2] = i.z;
    }
}
