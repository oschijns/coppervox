//!
//!
//!

/// Binds with Godot to run the logic circuit using a compute shader
mod runner;

/// Read quads data to generate a mesh usable by Godot
mod quads_reader;

use godot::prelude::*;

/// CopperVox as a Rust extension for godot
struct CopperVoxExtension;

#[gdextension]
unsafe impl ExtensionLibrary for CopperVoxExtension {}
