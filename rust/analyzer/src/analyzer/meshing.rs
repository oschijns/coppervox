use super::{QuadsReader, ShapeData};
use block_mesh::{
    GreedyQuadsBuffer, MergeVoxel, QuadCoordinateConfig, RIGHT_HANDED_Y_UP_CONFIG, Voxel,
    VoxelVisibility, greedy_quads,
};
use itertools::iproduct;
use ndshape::RuntimeShape;

/// Read shape's data and generate a mesh made of triangles from it
pub struct Mesher {
    /// define how to flip faces to avoid incorrect texture mirroring
    pub config: QuadCoordinateConfig,

    /// Work buffer of cells to process with the block_mesh crate
    work_buffer: Vec<BoolVoxel>,

    /// Buffer for storing the generated quad data
    quad_buffer: GreedyQuadsBuffer,
}

impl Mesher {
    pub fn build_mesh<R, M>(&mut self, shape: &ShapeData, reader: &mut R) -> M
    where
        R: QuadsReader<M>,
    {
        // To build a mesh with block mesh, we need to allocate a shell of
        // empty cells around the model.

        // Get the current dimensions of the shape and deduce the dimensions of
        // the work buffer to allocate.
        let [dix, diy, diz] = shape.dimensions;
        let dox = dix + 2;
        let doy = diy + 2;
        let doz = diz + 2;

        // reallocate the work buffer
        self.work_buffer.clear();
        self.work_buffer
            .resize(dox * doy * doz, BoolVoxel::default());

        // Read from the bitmatrix and write to the work buffer
        for (ix, iy, iz) in iproduct!(0..dix, 0..diy, 0..diz) {
            // index to read in the bitmatrix
            let index_in = ix + dix * (iy + diy * iz);

            // index to write in the work buffer
            let index_out = (ix + 1) + dox * ((iy + 1) + doy * (iz + 1));

            // Set the voxel value in the work buffer
            self.work_buffer[index_out] = BoolVoxel(shape.bitmatrix[index_in]);
        }

        // The work buffer can be used to generate quads now.
        let area = [dox as u32, doy as u32, doz as u32];
        let runtime_shape = RuntimeShape::<u32, 3>::new(area);
        greedy_quads(
            &self.work_buffer,
            &runtime_shape,
            [0, 0, 0],
            area,
            &self.config.faces,
            &mut self.quad_buffer,
        );

        // We can then extract the quad data as indices, positions and normals
        reader.read_quads(&self.quad_buffer.quads, &self.config)
    }
}

impl Default for Mesher {
    /// Default configuration assumed to be used with Godot or Bevy
    #[inline]
    fn default() -> Self {
        Self::new(RIGHT_HANDED_Y_UP_CONFIG, 0)
    }
}

impl Mesher {
    /// Create a new mesher with the provided quad coordinate configuration.
    /// If unsure, the capacity parameter can be set to zero.
    #[inline]
    pub fn new(config: QuadCoordinateConfig, capacity: usize) -> Self {
        Self {
            config,
            work_buffer: Vec::with_capacity(capacity),
            quad_buffer: GreedyQuadsBuffer::new(capacity),
        }
    }
}

/// Define a boolean which can implement block_mesh traits
#[derive(Default, Clone, Copy, Eq, PartialEq)]
struct BoolVoxel(bool);

impl Voxel for BoolVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if self.0 {
            VoxelVisibility::Opaque
        } else {
            VoxelVisibility::Empty
        }
    }
}

impl MergeVoxel for BoolVoxel {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

impl From<bool> for BoolVoxel {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<BoolVoxel> for bool {
    fn from(value: BoolVoxel) -> Self {
        value.0
    }
}
