use kiss3d::prelude::*;
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

#[kiss3d::main]
async fn main() {
    let mut window = Window::new("CopperVox").await;

    let mut camera = OrbitCamera3d::new(Vec3::new(0.0, 0.0, 6.0), Vec3::ZERO);
    let mut scene = SceneNode3d::empty();
    scene
        .add_light(Light::point(100.0))
        .set_position(Vec3::new(0.0, 10.0, 10.0));

    let rot = Quat::from_axis_angle(Vec3::Y, 0.014);

    while window.render_3d(&mut scene, &mut camera).await {
        //c.rotate(rot);
    }
}
