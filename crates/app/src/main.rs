/// Import voxel models
mod import;

/// Construct the circuit and mesh
mod construct;

use crate::{construct::Construct, import::load_config};
use clap::Parser;
use kiss3d::{
    event::{Action, WindowEvent},
    prelude::*,
};
use std::{path::PathBuf, process::exit};

/// Command line interface
#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to the configuration file
    #[arg(value_name = "FILE")]
    config: PathBuf,

    /// Render the generated circuit mesh with a standard material instead of the Runner shader
    #[arg(long)]
    preview_mesh: bool,
}

#[kiss3d::main]
async fn main() {
    let cli = Cli::parse();
    let mut config = load_config(&cli.config).expect("Config file should exists");
    config.path = cli
        .config
        .parent()
        .expect("Config file should have a parent directory")
        .to_path_buf();

    // Build the circuit first
    let construct = match Construct::new(&config) {
        Ok(construct) => construct,
        Err(error) => {
            eprintln!("Failed to load the model:\n{}", error);
            exit(1);
        }
    };

    println!("Model center: {}", construct.center);

    // Setup the rendering of the scene
    let mut window = Window::new("CopperVox").await;
    let mut camera =
        OrbitCamera3d::new(construct.center + construct.camera_offset, construct.center);
    let mut scene = SceneNode3d::empty();
    scene
        .add_light(Light::directional(Vec3::NEG_Y))
        .set_position(Vec3::ZERO);

    // Preview the generated geometry independently of the custom Runner pipeline.
    if cli.preview_mesh {
        println!(
            "Preview mesh: {} vertices, {} triangles",
            construct.preview_mesh.coords.len(),
            construct.preview_mesh.num_triangles()
        );
        scene
            .add_render_mesh(construct.preview_mesh, Vec3::ONE)
            .set_color(MAGENTA);
    } else {
        scene
            .add_render_mesh(construct.render_mesh, Vec3::ONE)
            .set_material(construct.material);
    }

    // main loop
    while window.render_3d(&mut scene, &mut camera).await {
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(key, Action::Press, _) => {
                    construct.pressed_keys.borrow_mut().insert(key);
                }
                WindowEvent::Key(key, Action::Release, _) => {
                    construct.pressed_keys.borrow_mut().remove(&key);
                }
                WindowEvent::Focus(false) => construct.pressed_keys.borrow_mut().clear(),
                _ => {}
            }
        }
    }
}
