/// Import voxel models
mod import;

/// Construct the circuit and mesh
mod construct;

use crate::construct::Construct;
use clap::Parser;
use kiss3d::prelude::*;
use std::{path::PathBuf, process::exit};

/// Command line interface
#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    model: PathBuf,

    #[arg(short, long)]
    config: PathBuf,
}

#[kiss3d::main]
async fn main() {
    let cli = Cli::parse();

    // Build the circuit first
    let construct = match Construct::new(&cli) {
        Ok(construct) => construct,
        Err(error) => {
            eprintln!("Failed to load the model:\n{}", error);
            exit(1);
        }
    };

    println!("Model center: {}", construct.center);

    // Setup the rendering of the scene
    let mut window = Window::new("CopperVox").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(0.0, 20.0, 20.0), Vec3::ZERO);
    let mut scene = SceneNode3d::empty();
    scene
        .add_light(Light::directional(Vec3::NEG_Y))
        .set_position(Vec3::ZERO);

    // Supply the circuit to the scene
    scene
        .add_render_mesh(construct.render_mesh, Vec3::ONE)
        .set_material(construct.material)
        .set_position(-construct.center);

    // main loop
    while window.render_3d(&mut scene, &mut camera).await {}
}
