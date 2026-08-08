/// Define constants as defined in the shader
pub mod shader_const;

/// Loader for building logic circuits
pub mod loader;

/// Run the circuit logic
pub mod runner;

// re-export crates
pub use coppervox_analyzer as analyzer;
pub use coppervox_common as common;
pub use coppervox_config as config;
