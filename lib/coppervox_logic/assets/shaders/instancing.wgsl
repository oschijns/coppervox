#import bevy_pbr::forward_io::{Vertex, VertexOutput}


// https://github.com/bevyengine/bevy/blob/a9ca8491aa3f2dc99116e63d67a0625afe6e738d/crates/bevy_pbr/src/render/forward_io.wgsl
// https://github.com/bevyengine/bevy/blob/a9ca8491aa3f2dc99116e63d67a0625afe6e738d/crates/bevy_pbr/src/render/mesh.wgsl


// https://github.com/bevyengine/bevy/blob/main/examples/shader/shader_instancing.rs
// https://github.com/bevyengine/bevy/blob/main/assets/shaders/instancing.wgsl


#define VERTEX_POSITIONS
#define VERTEX_NORMALS
#define VERTEX_TANGENTS
#define VERTEX_COLORS

// [!] we need this option for the instance_index to be transmitted to the fragment shader
#define VERTEX_OUTPUT_INSTANCE_INDEX


// colors of the component based on the value transmitted through the component
@group(2) @binding(0) var<uniform> color_off  : vec4<f32>;
@group(2) @binding(1) var<uniform> color_low  : vec4<f32>;
@group(2) @binding(2) var<uniform> color_high : vec4<f32>;

// base albedo of the material
@group(2) @binding(3) var albedo_scaling : vec2<f32>;
@group(2) @binding(4) var albedo_texture : texture_2d<f32>;
@group(2) @binding(5) var albedo_sampler : sampler;

// normal map of the material
@group(2) @binding(6) var normal_scaling : vec2<f32>;
@group(2) @binding(7) var normal_texture : texture_2d<f32>;
@group(2) @binding(8) var normal_sampler : sampler;


// buffer of gradients for each instance
#ifdef PER_OBJECT_BUFFER_BATCH_SIZE
@group(3) @binding(0) var<uniform> gradients: array<f32, #{PER_OBJECT_BUFFER_BATCH_SIZE}u>;
#else
@group(3) @binding(0) var<storage> gradients: array<f32>;
#endif // PER_OBJECT_BUFFER_BATCH_SIZE


// threshold of the normal to select the UV to use
const NORMAL_UV_THRESHOLD: f32 = 0.5;


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {

    // select the UVs based on 
    // - the normal of the fragment 
    // - the position of the mesh in the world
    var uv = vec2<f32>(0.0, 0.0);

    // check each axis of the normal, 
    // pick the axes orthonormal to the one with the biggest magnitude
    let abs_normal = abs(in.normal);
    if abs_normal.x > NORMAL_UV_THRESHOLD {
        uv = in.world_position.yz;
    } else if abs_normal.y > NORMAL_UV_THRESHOLD {
        uv = in.world_position.xz;
    } else if abs_normal.z > NORMAL_UV_THRESHOLD {
        uv = in.world_position.xy;
    }

    // from the UV, pick the albedo and the normal to use for this fragment
    let albedo = textureSample(albedo_texture, albedo_sampler, uv * albedo_scaling) * in.color;
    let normal = textureSample(normal_texture, normal_sampler, uv * normal_scaling) * in.normal;

    // get the gradient to use for this instance
    let gradient = gradients[in.instance_index];

    // pick the color based on the gradient
    var color = color_off;
    if gradient > 0.0 {
        color = smoothstep(color_low, color_high, gradient);
    }

    return color;
}
