//!
//!
//!

/// Binds with Godot to run the logic circuit using a compute shader
mod runner;

/// Analyze the voxel model and generate a logical representation that can be run by Godot
mod builder;

use godot::prelude::*;

/// CopperVox as a Rust extension for godot
struct CopperVoxExtension;

#[gdextension]
unsafe impl ExtensionLibrary for CopperVoxExtension {}
