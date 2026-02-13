// Render the components in the scene

// Define the data related to a component
struct ColorsUniform {
    off  : vec4<f32>,
    low  : vec4<f32>,
    high : vec4<f32>,
}

// Projection matrix to apply to object for rendering them
@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

/// Index of the value to pull from the storage buffer
@group(1) @binding(0)
var<uniform> index: u32;

// The three colors to use for lighting up the component upon activation
@group(1) @binding(1)
var<uniform> colors: ColorsUniform;

/// Transform of the mesh in the scene
@group(1) @binding(2)
var<uniform> transform: mat4x4<f32>;

// Processed values from the compute shader
@group(2) @binding(0)
var<storage, read> processed: array<u32>;


// Maximum value possible in a component
const MAX   = ~0u;
const RANGE = f32(MAX);


// Vertex input
struct ToVertex {
    @location(0) position : vec3<f32>,
    @location(1) normal   : vec3<f32>,
    @location(2) index    : u32,
}

// Vertex output / Fragment input
struct ToFragment {
    @builtin(position) position : vec4<f32>,
    @location(0)       normal   : vec4<f32>,
    @location(1)       color    : vec4<f32>,
}

// Process the vertices of the mesh
@vertex
fn vertex_main(vertex: ToVertex) -> ToFragment {
    var out: ToFragment;

    // Position the vertex relative to the camera
    let trs = projection * transform;
    out.position = trs * vec4<f32>(vertex.position, 1.0);
    out.normal   = trs * vec4<f32>(vertex.normal  , 0.0);

    // Get the value of the component to deduce its color
    let value = processed[index];
    switch value {
        case 0u  : { out.color = colors.off;  }
        case 1u  : { out.color = colors.low;  }
        case MAX : { out.color = colors.high; }
        default: {
            let t = f32(value) / RANGE;
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
