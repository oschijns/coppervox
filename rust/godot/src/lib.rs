//!
//!
//!

/// Binds with Godot to run the logic circuit using a compute shader
mod runner;

/// Analyze the voxel model and generate a logical representation that can be run by Godot
mod builder;

/// Convert a Godot's GridMap into a model to be processed
mod converter;

use godot::prelude::*;

/// Index of a component in the buffer
type CompIndex = usize;

/// Value of a component in the buffer
type CompValue = u32;

/// Define type use to encode operation type
pub type CompOp = i32;

/// CopperVox as a Rust extension for godot
struct CopperVoxExtension;

#[gdextension]
unsafe impl ExtensionLibrary for CopperVoxExtension {}
