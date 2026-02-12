//!

/// Common data between Rust and GLSL shaders
pub mod base;

/// Loader for building logic circuits
pub mod loader;

/// Run the circuit logic
pub mod runner;

use std::path::PathBuf;

/// Command line interface
#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    model: PathBuf,

    #[arg(short, long)]
    config: PathBuf,
}

#[macroquad::main("CopperVox")]
async fn main() {
    //let args = cli::Cli::parse();

    loop {}
}
