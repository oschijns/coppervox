#[compute]
#version 460


// Specify the size of integers used in this shader
const uint INT_SIZE = 32;
const uint INT_BIT  =  5; // 2^5 = 32

const uint FILLED = 0xffffffff;

// List of operations supported
const uint OP_OR   = 0;
const uint OP_AND  = 1;
const uint OP_NOR  = 2;
const uint OP_NAND = 3;
const uint OP_ADD  = 4;
const uint OP_MUL  = 5;
const uint OP_MIN  = 6;
const uint OP_MAX  = 7;


// Use 256 local invocations per workgroup
layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;


// Specify the number of components to handle.
// Following provided buffers should match this size accordingly.
layout(set = 0, binding = 0, std430) restrict readonly buffer Size {
    // Number of components in the simulation
    uint size;
};


// Connection matrix of the components.
layout(set = 0, binding = 1, std430) restrict readonly buffer Connections {
    // Store a single bit for each input-output connections.
    // This buffer should be `(size ^ 2 + 1) / 32`.
    // Accessing this buffer should be equivalent to [out][in] for optimization purposes.
    uint connections[];
};


// Specify the types of the output components indicating which operation to execute.
layout(set = 0, binding = 2, std430) restrict readonly buffer Operations {
    // Specify the operation to execute for each output.
    // This buffer should be `size` long.
    uint operations[];
};


// The input components
layout(set = 0, binding = 3, std430) readonly buffer Inputs {
    // Store a 32-bits unsigned integer for the state of the inputs.
    // This buffer should be `size` long.
    uint inputs[];
};


// The output components
layout(set = 0, binding = 4, std430) writeonly buffer Outputs {
    // Store a 32-bits unsigned integer for the state of the ouputs.
    // This buffer should be `size` long.
    uint outputs[];
};


// Read the connection buffer to figure out if the input should be used
bool use_input(uint output_idx, uint input_idx) {
    uint bit_pos = output_idx * size + input_idx;
    
    // Calculate which word and bit within that word
    uint word_idx = bit_pos >> INT_BIT;        // Divide by 32
    uint bit_idx = bit_pos & (INT_SIZE - 1);   // Modulo 32
    
    // Get the word and check the bit
    uint word = connections[word_idx];
    return ((word >> bit_idx) & 1u) != 0u;
}


// Process OR operation
uint op_or(uint gid, uint offset) {
    uint result = 0;
    for (uint i = 0; i < size; ++i) {
        if (i != gid && use_input(offset, i)) {
            result |= inputs[i];
        }
    }
    return result;
}


// Process AND operation
uint op_and(uint gid, uint offset) {
    uint result = FILLED;
    for (uint i = 0; i < size; ++i) {
        if (i != gid && use_input(offset, i)) {
            result &= inputs[i];
        }
    }
    return result;
}


// Process ADD operation
uint op_add(uint gid, uint offset) {
    uint result = 0;
    for (uint i = 0; i < size; ++i) {
        if (i != gid && use_input(offset, i)) {
            result += inputs[i];
        }
    }
    return result;
}


// Process MUL operation
uint op_mul(uint gid, uint offset) {
    uint result = 1;
    for (uint i = 0; i < size; ++i) {
        if (i != gid && use_input(offset, i)) {
            result *= inputs[i];
        }
    }
    return result;
}


// Process MIN operation
uint op_min(uint gid, uint offset) {
    uint result = FILLED;
    for (uint i = 0; i < size; ++i) {
        if (i != gid && use_input(offset, i)) {
            result = min(result, inputs[i]);
        }
    }
    return result;
}


// Process MAX operation
uint op_max(uint gid, uint offset) {
    uint result = 0;
    for (uint i = 0; i < size; ++i) {
        if (i != gid && use_input(offset, i)) {
            result = max(result, inputs[i]);
        }
    }
    return result;
}


void main() {
	// Global output index (one invocation per output element)
	uint gid = gl_GlobalInvocationID.x;

	// If out-of-bounds, do nothing
	if (gid >= size) return;

    // Compute the offset to use to access the connection matrix.
    uint offset = gid * (size + 1) >> INT_BIT; // divide by INT_SIZE

    // Result to store in the output buffer
    uint result;
    switch (operations[gid]) {
        case OP_OR  : result =  op_or (gid, offset); break;
        case OP_AND : result =  op_and(gid, offset); break;
        case OP_NOR : result = ~op_or (gid, offset); break;
        case OP_NAND: result = ~op_and(gid, offset); break;
        case OP_ADD : result =  op_add(gid, offset); break;
        case OP_MUL : result =  op_mul(gid, offset); break;
        case OP_MIN : result =  op_min(gid, offset); break;
        case OP_MAX : result =  op_max(gid, offset); break;
        default: result = 0; break;
    }
    outputs[gid] = result;
}