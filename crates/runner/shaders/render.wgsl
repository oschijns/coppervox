// Render the components in the scene

/*
 * Structures are expected to be aligned over 16-bytes
 * which correspond to a `vec4<u32>`.
 */

// Per-object data structure
struct InstanceData {
    /// Translation to apply to the mesh
    translation: vec3<f32>,

    /// Index of the color set to pull from the colors buffer
    colors_index: u32,
}

// Define the data related to a component
struct ColorPreset {
    /// Color used when the component is fully off (0)
    off: vec4<f32>,

    /// Color used when the component has low power (1)
    low: vec4<f32>,

    /// Color used when the component has high power (u32::MAX)
    high: vec4<f32>,
}

// Projection matrix to apply to object for rendering them
@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

/// Storage buffer containing all object instances
@group(1) @binding(0)
var<storage, read> instances: array<InstanceData>;

// The three colors to use for lighting up the component upon activation
@group(1) @binding(1)
var<storage, read> color_presets: array<ColorPreset>;

// Processed values from the compute shader
@group(2) @binding(0)
var<storage, read> processed: array<u32>;

// Vertex input
struct ToVertex {
    @location(0) position: vec3<f32>,
    @location(1) normal  : vec3<f32>,

    // The UV data stores the render-instance and processing-buffer indexes.
    @location(2) uv: vec2<u32>,
}

// Vertex output / Fragment input
struct ToFragment {
    @builtin(position) position: vec4<f32>,
    @location(0)       normal  : vec4<f32>,
    @location(1)       color   : vec4<f32>,
}

// Process the vertices of the mesh
@vertex
fn vertex_main(vertex: ToVertex) -> ToFragment {
    var out: ToFragment;

    // Get per-instance data
    let comp = instances[vertex.uv.x];

    // Position the vertex relative to the camera
    let trs = projection;
    out.position = trs * vec4<f32>(comp.translation + vertex.position, 1.0);
    out.normal   = trs * vec4<f32>(vertex.normal, 0.0);

    // Maximum possible value for a component
    const MAX: u32 = ~0u;

    // Get the value of the component to deduce its color
    let value = processed[vertex.uv.y];
    let colors = color_presets[comp.colors_index];
    switch value {
        case  0u: { out.color = colors.off;  }
        case  1u: { out.color = colors.low;  }
        case MAX: { out.color = colors.high; }
        default: {
            let t = f32(value) / f32(MAX);
            out.color = smoothstep(colors.low, colors.high, vec4<f32>(t, t, t, t));
        }
    }

    return out;
}

// Compute the fragments to render
@fragment
fn fragment_main(fragment: ToFragment) -> @location(0) vec4<f32> {
    // TODO: compute shading effect ?
    return fragment.color;
}
