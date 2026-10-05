//! Hand-builds a tiny logic circuit directly (bypassing the voxel model /
//! analyzer / config pipeline) so the `Runner` compute+render path can be
//! validated in isolation: two constant inputs are OR'd into one output.
//!
//!   component 0: constant input, always on  (u32::MAX)
//!   component 1: constant input, always off (0)
//!   component 2: OR(component 0, component 1) -> always on
//!
//! Each component is drawn as its own cube (left to right); the middle one
//! should render dark ("off") while the two side ones render bright ("on").

use coppervox_runner::{
    runner::{Runner, RunnerData},
    shader_const::{GpuColorGradient, GpuInstanceData, GpuMat4x4, GpuOp, MAX_CNX},
};
use kiss3d::{
    prelude::*,
    procedural::{IndexBuffer, RenderMesh},
};

#[kiss3d::main]
async fn main() {
    let mut window = Window::new("CopperVox: simple circuit").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(0.0, 2.0, 6.0), Vec3::ZERO);
    let mut scene = SceneNode3d::empty();
    scene
        .add_light(Light::point(100.0))
        .set_position(Vec3::new(0.0, 10.0, 10.0));

    scene
        .add_render_mesh(build_mesh(), Vec3::new(1.0, 1.0, 1.0))
        .set_material(build_material());

    while window.render_3d(&mut scene, &mut camera).await {}
}

/// Build the `Runner` material from a hand-written 3-component circuit
fn build_material() -> std::rc::Rc<std::cell::RefCell<Box<dyn Material3d + 'static>>> {
    let cnx = MAX_CNX as usize;
    let mut connections = vec![0u32; 3 * cnx];
    connections[2 * cnx] = 0; // component 2, input 0 -> component 0
    connections[2 * cnx + 1] = 1; // component 2, input 1 -> component 1

    let operations = vec![
        0,                      // component 0: no-op, just passes its own value through
        0,                      // component 1: no-op, just passes its own value through
        (GpuOp::Or as u32) | 2, // component 2: OR of its 2 connections
    ];

    // Seeds both ping and pong buffers: the constants stay stable every
    // tick since their `count` is 0, so the compute shader just echoes them.
    let inits = vec![u32::MAX, 0, 0];

    let instances_data = vec![
        GpuInstanceData::new([-1.5, 0.0, 0.0], 0),
        GpuInstanceData::new([0.0, 0.0, 0.0], 0),
        GpuInstanceData::new([1.5, 0.0, 0.0], 0),
    ];

    let color_presets = vec![GpuColorGradient::new(
        [0.05, 0.05, 0.05, 1.0], // off: dark gray
        [0.6, 0.55, 0.05, 1.0],  // low: amber (unused by this all-or-nothing circuit)
        [0.15, 0.9, 0.25, 1.0],  // high: green
    )]
    .into_boxed_slice();

    // Overwritten by `Runner::prepare()` on the very first frame; only
    // needs to be a well-formed matrix until then.
    let projection: GpuMat4x4 = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    let data = RunnerData::new(
        projection,
        connections.into_boxed_slice(),
        operations.into_boxed_slice(),
        instances_data.into_boxed_slice(),
        color_presets,
        inits.into_boxed_slice(),
    );

    Rc::new(RefCell::new(
        Box::new(Runner::new(data)) as Box<dyn Material3d + 'static>
    ))
}

/// Build one combined mesh holding one cube per component. Instance
/// translation and color are *not* baked into the cube coordinates: they
/// are looked up on the GPU from the `uv` tag (see `render.wgsl`).
fn build_mesh() -> RenderMesh {
    let mut coords = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for component in 0..3u32 {
        push_cube(&mut coords, &mut normals, &mut uvs, &mut indices, component);
    }

    RenderMesh::new(
        coords,
        Some(normals),
        Some(uvs),
        Some(IndexBuffer::Unified(indices)),
    )
}

/// Push one unit cube centered at the origin into the shared mesh buffers.
/// Every vertex is tagged `[component, component]`: the render shader uses
/// the first index to look up this component's transform/color and the
/// second to look up its simulated value.
fn push_cube(
    coords: &mut Vec<Vec3>,
    normals: &mut Vec<Vec3>,
    uvs: &mut Vec<Vec2>,
    indices: &mut Vec<[u32; 3]>,
    tag: u32,
) {
    const H: f32 = 0.4;
    #[rustfmt::skip]
    let faces: [([[f32; 3]; 4], [f32; 3]); 6] = [
        ([[ H,-H,-H], [ H, H,-H], [ H, H, H], [ H,-H, H]], [ 1.0,  0.0,  0.0]),
        ([[-H,-H, H], [-H, H, H], [-H, H,-H], [-H,-H,-H]], [-1.0,  0.0,  0.0]),
        ([[-H, H,-H], [-H, H, H], [ H, H, H], [ H, H,-H]], [ 0.0,  1.0,  0.0]),
        ([[-H,-H, H], [-H,-H,-H], [ H,-H,-H], [ H,-H, H]], [ 0.0, -1.0,  0.0]),
        ([[-H,-H, H], [ H,-H, H], [ H, H, H], [-H, H, H]], [ 0.0,  0.0,  1.0]),
        ([[ H,-H,-H], [-H,-H,-H], [-H, H,-H], [ H, H,-H]], [ 0.0,  0.0, -1.0]),
    ];

    // The UV data is hacked to store the two `u32` indices, bit-reinterpreted
    // as `f32` so they fit kiss3d's regular UV buffer (see `render.wgsl`).
    let uv = Vec2::new(f32::from_bits(tag), f32::from_bits(tag));
    for (corners, normal) in faces {
        let base = coords.len() as u32;
        let n = Vec3::new(normal[0], normal[1], normal[2]);
        for c in corners {
            coords.push(Vec3::new(c[0], c[1], c[2]));
            normals.push(n);
            uvs.push(uv);
        }
        indices.push([base, base + 1, base + 2]);
        indices.push([base, base + 2, base + 3]);
    }
}
