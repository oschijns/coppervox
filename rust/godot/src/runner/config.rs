/// Size of a workgroup in the compute shader
pub const WORKGROUP_SIZE: u32 = 256;

/// Binding point for the number of components
pub const BIND_SIZE: i32 = 0;

/// Binding point for the incidence matrix
pub const BIND_CONNECTIONS: i32 = 1;

/// Binding point for the types of the components
pub const BIND_OPERATIONS: i32 = 2;

/// Binding point for the inputs
pub const BIND_INPUTS: i32 = 3;

/// Binding point for the outputs
pub const BIND_OUTPUTS: i32 = 4;

/// Binding point for the output sampler
pub const BIND_SAMPLER: i32 = 5;
