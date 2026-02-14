
// Define the size of the workgroup
const WORKGROUP = 256;


// Define the maximum number of connexions per output.
const MAX_CNX = 16;

// Define connections between outputs and inputs.
// The storage array must be sized as (output_count * MAX_CNX).
@group(0) @binding(0)
var<storage, read> connections: array<u32>;


// List of operations supported
const OP_OR   = 1 << 24; ///< bitwise OR   operation
const OP_AND  = 2 << 24; ///< bitwise AND  operation
const OP_NOR  = 3 << 24; ///< bitwise NOR  operation
const OP_NAND = 4 << 24; ///< bitwise NAND operation
const OP_ADD  = 5 << 24; ///< addition
const OP_MUL  = 6 << 24; ///< multiplication
const OP_MIN  = 7 << 24; ///< minimum
const OP_MAX  = 8 << 24; ///< maximum

// Define the type of operation and the number of inputs to read for each output.
// The most significant byte store the operation type.
// The remaining three bytes store the number of inputs to read.
@group(0) @binding(1)
var<storage, read> operations: array<u32>;


// Input values to process
@group(1) @binding(0)
var<storage, read> inputs: array<u32>;


// Ouput values to write
@group(1) @binding(1)
var<storage, read_write> outputs: array<u32>;


// Process OR operation
fn op_or(offset: u32, count: u32) -> u32 {
    var result: u32 = 0u;
    for (var i = 0u; i < min(count, MAX_CNX); i++) {
        result |= inputs[connections[offset + i]];
    }
    return result;
}

// Process OR operation
fn op_and(offset: u32, count: u32) -> u32 {
    var result: u32 = ~0u;
    for (var i = 0u; i < min(count, MAX_CNX); i++) {
        result &= inputs[connections[offset + i]];
    }
    return result;
}

// Process OR operation
fn op_add(offset: u32, count: u32) -> u32 {
    var result: u32 = 0u;
    for (var i = 0u; i < min(count, MAX_CNX); i++) {
        result += inputs[connections[offset + i]];
    }
    return result;
}

// Process OR operation
fn op_mul(offset: u32, count: u32) -> u32 {
    var result: u32 = 1u;
    for (var i = 0u; i < min(count, MAX_CNX); i++) {
        result *= inputs[connections[offset + i]];
    }
    return result;
}

// Process OR operation
fn op_min(offset: u32, count: u32) -> u32 {
    var result: u32 = ~0u;
    for (var i = 0u; i < min(count, MAX_CNX); i++) {
        result = min(result, inputs[connections[offset + i]]);
    }
    return result;
}

// Process OR operation
fn op_max(offset: u32, count: u32) -> u32 {
    var result: u32 = 0u;
    for (var i = 0u; i < min(count, MAX_CNX); i++) {
        result = max(result, inputs[connections[offset + i]]);
    }
    return result;
}


// Read the inputs and compute the value to store in the output buffer.
@compute @workgroup_size(WORKGROUP, 1, 1)
fn process(
    // identifier for the output value to compute
    @builtin(global_invocation_id)
    id: vec3<u32>,
) {
    // Check that we are not going out-of-bound
    if id.x < min(arrayLength(&inputs), arrayLength(&outputs)) {

        // Where to start reading the `connections` buffer ?
        let offset = id.x * MAX_CNX;

        // How many 32-bits word to read in the sequence ?
        let op_data = operations[id.x];
        let count = op_data & 0xFFFFFF;

        // Compute the new value to store in the output buffer
        var result = 0u;

        // We need at least one input to do actual processing
        if count > 0u {
            switch op_data & 0xFF000000 {
                case OP_OR  : { result =  op_or (offset, count); }
                case OP_AND : { result =  op_and(offset, count); }
                case OP_NOR : { result = ~op_or (offset, count); }
                case OP_NAND: { result = ~op_and(offset, count); }
                case OP_ADD : { result =  op_add(offset, count); }
                case OP_MUL : { result =  op_mul(offset, count); }
                case OP_MIN : { result =  op_min(offset, count); }
                case OP_MAX : { result =  op_max(offset, count); }
                default     : { result = inputs[id.x]; }
            }
        } else {
            result = inputs[id.x];
        }

        // Store the result in the output buffer
        outputs[id.x] = result;
    }
}
