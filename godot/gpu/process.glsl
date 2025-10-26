#[compute]
#version 460

// Use inputs to compute outputs values based on connections and output type.
// 


// Specify the size of integers used in this shader
const uint INT_SIZE = 32;
const uint INT_BIT  =  5; // 2^5 = 32
const uint INT_MASK = INT_SIZE - 1;

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


// Specify the number of inputs and the number of ouputs to handle.
// Following provided buffers should match those size accordingly.
layout(std430, bindings = 0) uniform Dimensions {
    // Number of input components
    uint nb_inputs;
    // Number of ouput components
    uint nb_outputs;
};


// Connection matrix of the components.
layout(std430, binding = 1) restrict readonly buffer Connections {
    // Store a single bit for each input-output connections.
    // This buffer should be `(nb_outputs * nb_inputs + 1) / 32`.
    // Accessing this buffer should be equivalent to [out][in] for optimization purposes.
    uint connections[];
};


// Specify the types of the output components indicating which operation to execute.
layout(std430, binding = 2) restrict readonly buffer Operations {
    // Specify the operation to execute for each output.
    // This buffer should be `nb_outputs` long.
    uint operations[];
};


// The input components
layout(std430, binding = 3) readonly buffer Inputs {
    // Store a 32-bits unsigned integer for the state of the inputs.
    // This buffer should be `nb_inputs` long.
    uint inputs[];
};


// The output components
layout(std430, binding = 4) writeonly buffer Outputs {
    // Store a 32-bits unsigned integer for the state of the ouputs.
    // This buffer should be `nb_outputs` long.
    uint outputs[];
};


void main() {
	// Global output index (one invocation per output element)
	uint gid = gl_GlobalInvocationID.x;

	// If out-of-bounds, do nothing
	if (gid >= nb_outputs) return;

    // Compute the offset to use to access the connection matrix.
    uint offset = gid * (nb_inputs + 1) >> INT_BIT; // divide by INT_SIZE

    // Result to store in the output buffer
    uint result;
    switch (operations[gid]) {
        case OP_OR  : result = op_or  (offset); break;
        case OP_AND : result = op_and (offset); break;
        case OP_NOR : result = op_nor (offset); break;
        case OP_NAND: result = op_nand(offset); break;
        case OP_ADD : result = op_add (offset); break;
        case OP_MUL : result = op_mul (offset); break;
        case OP_MIN : result = op_min (offset); break;
        case OP_MAX : result = op_max (offset); break;
        default: result = 0; break;
    }
    outputs[gid] = result;
}


// Process OR operation
uint op_or(uint offset) {
    uint result = 0;
    for (uint i = 0; i < nb_inputs; ++i) {
        if (use_input(offset, i)) {
            result |= inputs[i];
        }
    }
    return result;
}


// Process AND operation
uint op_and(uint offset) {
    uint result = 0;
    for (uint i = 0; i < nb_inputs; ++i) {
        if (use_input(offset, i)) {
            result &= inputs[i];
        }
    }
    return result;
}


// Process NOR operation
uint op_nor(uint offset) {
    uint result = 0;
    for (uint i = 0; i < nb_inputs; ++i) {
        if (use_input(offset, i)) {
            result |= inputs[i];
        }
    }
    return ~result;
}


// Process NAND operation
uint op_nand(uint offset) {
    uint result = 0;
    for (uint i = 0; i < nb_inputs; ++i) {
        if (use_input(offset, i)) {
            result &= inputs[i];
        }
    }
    return ~result;
}


// Process ADD operation
uint op_add(uint offset) {
    uint result = 0;
    for (uint i = 0; i < nb_inputs; ++i) {
        if (use_input(offset, i)) {
            result += inputs[i];
        }
    }
    return result;
}


// Process MUL operation
uint op_mul(uint offset) {
    uint result = 0;
    for (uint i = 0; i < nb_inputs; ++i) {
        if (use_input(offset, i)) {
            result *= inputs[i];
        }
    }
    return result;
}


// Process MIN operation
uint op_min(uint offset) {
    uint result = 0;
    for (uint i = 0; i < nb_inputs; ++i) {
        if (use_input(offset, i)) {
            result = min(result, inputs[i]);
        }
    }
    return result;
}


// Process MAX operation
uint op_max(uint offset) {
    uint result = 0;
    for (uint i = 0; i < nb_inputs; ++i) {
        if (use_input(offset, i)) {
            result = max(result, inputs[i]);
        }
    }
    return result;
}


// Read the connection buffer to figure out if the input should be used
bool use_input(uint offset, uint index) {

    // split the index into word and bit
    //uint i_word = index / INT_SIZE;
    //uint i_bit  = index % INT_SIZE;
    uint i_word = index >> INT_BIT;
    uint i_bit  = index & (INT_SIZE - 1);

    // access the word
    uint word = connections[offset + i_word];

    // access the bit from the word
    return ((word >> i_bit) & 1) != 0;
}