use bevy::render::{
    mesh::{Indices, Mesh, VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};
use block_mesh::{
    greedy_quads, GreedyQuadsBuffer, MergeVoxel, QuadCoordinateConfig, Voxel, VoxelVisibility,
    RIGHT_HANDED_Y_UP_CONFIG,
};
use ndarray::Array3;
use ndshape::{RuntimeShape, Shape};

/// Matrix compatible with the `block_mesh` crate
pub struct BlockMeshMatrix<V> {
    /// serialized matrix cells
    data: Vec<V>,

    /// dimensions of the matrix
    shape: RuntimeShape<u32, 3>,

    /// define how to flip faces to avoid incorrect texture mirroring
    quads_config: QuadCoordinateConfig,
}

impl<V: MergeVoxel> BlockMeshMatrix<V> {
    /// Generate a mesh from an arbitrary voxel buffer
    pub fn build_mesh(&self, point_min: [u32; 3], point_max: [u32; 3]) -> Mesh {
        /* bevy_vox_mesh implementation by Lucas A.  */
        /* Adapted to the needs of this library      */
        /* https://github.com/Game4all/bevy_vox_mesh */

        // compute the boundaries of the mesh to generate taking into
        // account the one cell layer around the provided matrix
        let [ax, ay, az] = point_min;
        let [bx, by, bz] = point_max;
        let alloc_size = ((bx + by + bz) - (ax + ay + az)) as usize;

        // allocate a buffer to store the generated quads
        let mut greedy_quads_buffer = GreedyQuadsBuffer::new(alloc_size);
        greedy_quads(
            &self.data,
            &self.shape,
            [ax + 1, ay + 1, az + 1],
            [bx + 1, by + 1, bz + 1],
            &self.quads_config.faces,
            &mut greedy_quads_buffer,
        );

        let num_vertices = greedy_quads_buffer.quads.num_quads() * 4;
        let mut indices = Vec::<u32>::with_capacity(greedy_quads_buffer.quads.num_quads() * 6);
        let mut positions = Vec::<[f32; 3]>::with_capacity(num_vertices);
        let mut normals = Vec::<[f32; 3]>::with_capacity(num_vertices);

        let mut render_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        for (group, face) in greedy_quads_buffer
            .quads
            .groups
            .iter()
            .zip(self.quads_config.faces.as_ref())
        {
            for quad in group.iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(&face.quad_mesh_positions(quad, 1.0));
                normals.extend_from_slice(&face.quad_mesh_normals());
            }
        }

        render_mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        );

        render_mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        );

        render_mesh.insert_indices(Indices::U32(indices.clone()));

        render_mesh
    }
}

/// Convert a ndarray::Array3 into a BlockMeshMatrix
impl<V: Default + Clone> BlockMeshMatrix<V> {
    pub fn from_matrix<A: Copy + Into<V>>(matrix: &Array3<A>) -> Self {
        // A matrix used by the block mesh function needs
        // an one-cell extra shell around the initial matrix.
        let (sx, sy, sz) = matrix.dim();
        let shape = RuntimeShape::<u32, 3>::new([sx as u32 + 2, sy as u32 + 2, sz as u32 + 2]);

        // We need to allocate enough memory for this extra shell.
        let mut data = vec![V::default(); shape.size() as usize];

        // iterate of the matrix and store the data at the appropriate location in the data buffer
        for ((x, y, z), value) in matrix.indexed_iter() {
            let index = shape.linearize([x as u32 + 1, y as u32 + 1, z as u32 + 1]) as usize;
            data[index] = (*value).into();
        }

        Self {
            data,
            shape,
            quads_config: RIGHT_HANDED_Y_UP_CONFIG,
        }
    }
}

/// Define a boolean which can implement block_mesh traits
#[derive(Default, Clone, Copy, Eq, PartialEq)]
pub struct BoolVoxel(bool);

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
