use coppervox_common::{Model, Voxel, gridmap::ndarray::Array3};
use core::f32;
use glam::Vec3;
use xcf::{RgbaPixel, Xcf};

/// Number of values to remap
const SIZE: usize = 256;

/// Remap values from the XCF file
pub struct Remapper(Array3<Voxel>);

/// Create a default Remapper
impl Default for Remapper {
    /// Create a default empty remapper
    #[inline]
    fn default() -> Self {
        Self(Array3::zeros((SIZE, SIZE, SIZE)))
    }
}

impl Remapper {
    /// Create a remapper from a voronoid model
    pub fn from_voronoid(voronoid: &[(RgbaPixel, Voxel)]) -> Self {
        // Convert the array of colors to floating point values
        let points = voronoid
            .iter()
            .map(|(pixel, voxel)| (rgb_to_vec3(*pixel), *voxel))
            .collect::<Vec<_>>();

        // Create a RGB cube to pull values
        let mut matrix = Array3::<Voxel>::zeros((SIZE, SIZE, SIZE));

        // For each position in the cube try to find the closest value
        for (index, value) in matrix.indexed_iter_mut() {
            let pos = idx_to_vec3(index);
            let mut select_voxel = 0 as Voxel;
            let mut select_dist = f32::MAX;

            // find closest point in voronoid space
            for &(point, voxel) in points.iter() {
                let dist = pos.distance_squared(point);
                if dist < select_dist {
                    select_voxel = voxel;
                    select_dist = dist;
                }
            }

            // use the found voxel value and store it into the RGB cube
            *value = select_voxel;
        }

        Self(matrix)
    }

    /// Convert Magica Voxel data into a Model to be processed
    pub fn convert_gimp(&self, xcf: &Xcf, chunk_dim: [usize; 3]) -> Model {
        let mut canvas = Model::new(chunk_dim);

        for (w, layer) in xcf.layers.iter().enumerate() {
            let width = layer.width;
            let height = layer.height;

            for u in 0..width {
                for v in 0..height {
                    if let Some(pixel) = layer.pixel(u, v) {
                        let i = [u as isize, w as isize, v as isize];
                        canvas[i] = self.convert_voxel(pixel);
                    }
                }
            }
        }

        canvas
    }

    #[inline]
    fn convert_voxel(&self, pixel: RgbaPixel) -> Voxel {
        let i = [pixel.r() as usize, pixel.g() as usize, pixel.b() as usize];
        self.0[i]
    }
}

#[inline]
fn rgb_to_vec3(pixel: RgbaPixel) -> Vec3 {
    Vec3::new(pixel.r() as f32, pixel.g() as f32, pixel.b() as f32)
}

#[inline]
fn idx_to_vec3(index: (usize, usize, usize)) -> Vec3 {
    Vec3::new(index.0 as f32, index.1 as f32, index.2 as f32)
}
