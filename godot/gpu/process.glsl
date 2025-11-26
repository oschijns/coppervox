#[compute]
#version 460

// Define the maximum amount of connections per component
const uint MAX_CONN = 32;

// Specify the size of integers used in this shader
const uint INT_SIZE = 32;
const uint INT_BIT  =  5; // 2^5 = 32
const uint FILLED   = 0xffffffff;

// List of operations supported
const uint OP_OR   = 1;
const uint OP_AND  = 2;
const uint OP_NOR  = 3;
const uint OP_NAND = 4;
const uint OP_ADD  = 5;
const uint OP_MUL  = 6;
const uint OP_MIN  = 7;
const uint OP_MAX  = 8;


// Use 256 local invocations per workgroup
layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;

// Specify the number of components to handle.
// Following provided buffers should match this size accordingly.
layout(set = 0, binding = 0) restrict readonly uniform Size {
    // Number of components to read from
    uint size_in;

    // Number of components to write to
    uint size_out;
};

// Connection matrix of the components.
layout(set = 0, binding = 1, std430) restrict readonly buffer Connections {
    // Store a single bit for each input-output connections.
    // This buffer should be `size_out * MAX_CONN`.
    // Accessing this buffer should be equivalent to [out][in] for optimization purposes.
    uint connections[];
};

// Specify the types of the output components indicating which operation to execute.
layout(set = 0, binding = 2, std430) restrict readonly buffer Operations {
    // Specify the operation to execute for each output and the number of inputs to read from.
    // This buffer should be `size_out` long.
    uint operations[];
};

// The input components
layout(set = 1, binding = 0, std430) readonly buffer Inputs {
    // Store a 32-bits unsigned integer for the state of the inputs.
    // This buffer should be `size_in` long.
    uint inputs[];
};

// The output components
layout(set = 2, binding = 0, std430) writeonly buffer Outputs {
    // Store a 32-bits unsigned integer for the state of the ouputs.
    // This buffer should be `size_out` long.
    uint outputs[];
};

// The output image to write to
layout(set = 2, binding = 1, r32ui) writeonly uniform uimage1D write_output;


// Process OR operation
uint op_or(uint offset, uint count) {
    uint result = 0;
    for (uint i = 0; i < max(count, MAX_CONN); ++i) {
        result |= inputs[connections[offset + i]];
    }
    return result;
}

// Process AND operation
uint op_and(uint offset, uint count) {
    uint result = FILLED;
    for (uint i = 0; i < max(count, MAX_CONN); ++i) {
        result &= inputs[connections[offset + i]];
    }
    return result;
}

// Process ADD operation
uint op_add(uint offset, uint count) {
    uint result = 0;
    for (uint i = 0; i < max(count, MAX_CONN); ++i) {
        result += inputs[connections[offset + i]];
    }
    return result;
}

// Process MUL operation
uint op_mul(uint offset, uint count) {
    uint result = 1;
    for (uint i = 0; i < max(count, MAX_CONN); ++i) {
        result *= inputs[connections[offset + i]];
    }
    return result;
}

// Process MIN operation
uint op_min(uint offset, uint count) {
    uint result = FILLED;
    for (uint i = 0; i < max(count, MAX_CONN); ++i) {
        result = min(result, inputs[connections[offset + i]]);
    }
    return result;
}

// Process MAX operation
uint op_max(uint offset, uint count) {
    uint result = 0;
    for (uint i = 0; i < max(count, MAX_CONN); ++i) {
        result = max(result, inputs[connections[offset + i]]);
    }
    return result;
}

void main() {
    // Global output index (one invocation per output element)
    uint gid = gl_GlobalInvocationID.x;

    // If out-of-bounds, do nothing
    if (gid >= size_out) return;

    // Compute the offset to use to access the connection matrix.
    uint offset = gid * MAX_CONN;

    // Get the config of the component (operation + input count)
    uint data  = operations[gid];
    uint count = data >> 8;

    // Result to store in the output buffer
    uint result;
    switch (data & 0xff) {
        case OP_OR  : result =  op_or (offset, count); break;
        case OP_AND : result =  op_and(offset, count); break;
        case OP_NOR : result = ~op_or (offset, count); break;
        case OP_NAND: result = ~op_and(offset, count); break;
        case OP_ADD : result =  op_add(offset, count); break;
        case OP_MUL : result =  op_mul(offset, count); break;
        case OP_MIN : result =  op_min(offset, count); break;
        case OP_MAX : result =  op_max(offset, count); break;
        default: result = inputs[gid]; break;
    }
    outputs[gid] = result;
    imageStore(write_output, int(gid), uvec4(result));
}
