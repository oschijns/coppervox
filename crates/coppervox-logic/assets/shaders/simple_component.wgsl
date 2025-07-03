#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip
}

// buffer containing per instance data
@group(2) @binding(0) var<uniform>       instances_count   : u32;
@group(2) @binding(1) var<storage, read> instances_factors : array<f32>;

// colors of the component based on the value transmitted through
@group(3) @binding(0) var<uniform> color_off  : vec4<f32>;
@group(3) @binding(1) var<uniform> color_low  : vec4<f32>;
@group(3) @binding(2) var<uniform> color_high : vec4<f32>;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) color: vec4<f32>,
};


/// Apply a spherical linear interpolation between two colors
fn slerp(a : vec4<f32>, b : vec4<f32>, t : f32) -> vec4<f32> {
    // Normalize the vectors to unit length
    let a_norm = normalize(a);
    let b_norm = normalize(b);

    // Compute the dot product of the normalized vectors
    // Clamp the dot product to avoid numerical issues with arccos near -1/+1
    let dot_product = clamp(dot(a_norm, b_norm), -1.0, 1.0);

    // Compute the angle between the vectors
    let theta = acos(dot_product) * t;

    // Perform spherical linear interpolation
    let q = normalize(b_norm - dot_product * a_norm);
    return cos(theta) * a_norm + sin(theta) * q;
}


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4(vertex.position, 1.0));
    out.clip_position = position_world_to_clip(out.world_position.xyz);

    // each component has a different factor value
    var factor = instances_factors[vertex.instance_index % instances_count];

    // select the color to use based on the factor
    if factor >= 0.0 {
        out.color = slerp(color_low, color_high, factor);
    } else {
        out.color = color_off;
    }

    return out;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    return mesh.color;
}
