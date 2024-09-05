use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    render::render_asset::RenderAssetUsages,
    render::storage::ShaderStorageBuffer,
};

const SHADER_ASSET_PATH: &str = "shaders/simple_component.wgsl";

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut materials: ResMut<Assets<SimpleMaterial>>,
) {
    // Factors of each component to handle
    let count = 25;
    let factors = Vec::<f32>::with_capacity(count);

    let factors_buffer = buffers.add(ShaderStorageBuffer::new(
        bytemuck::cast_slice(factors.as_slice()),
        RenderAssetUsages::default(),
    ));

    // Create the custom material with the storage buffer
    let material = SimpleMaterial {
        instances_count: count as u32,
        instances_factors: factors_buffer,
    };

    let material_handle = materials.add(material);
    commands.insert_resource(SimpleMaterialHandle(material_handle.clone()));

    // Spawn cubes with the custom material
    /*
    for i in -6..=6 {
        for j in -3..=3 {
            commands.spawn(MaterialMeshBundle {
                mesh: meshes.add(Cuboid::from_size(Vec3::splat(0.3))),
                material: material_handle.clone(),
                transform: Transform::from_xyz(i as f32, j as f32, 0.0),
                ..default()
            });
        }
    }

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    */
}

// Update the material color by time
fn update(
    time: Res<Time>,
    material_handle: Res<SimpleMaterialHandle>,
    mut materials: ResMut<Assets<SimpleMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let material = materials.get_mut(&material_handle.0).unwrap();
    let buffer = buffers.get_mut(&material.instances_factors).unwrap();
    buffer.data = Some(
        bytemuck::cast_slice(
            (0..25)
                .map(|i| {
                    let t = time.elapsed_seconds() * 5.0;
                    (t + i as f32).sin() / 2.0 + 0.5
                })
                .collect::<Vec<f32>>()
                .as_slice(),
        )
        .to_vec(),
    );
}

// Holds a handle to the custom material
#[derive(Resource)]
struct SimpleMaterialHandle(Handle<SimpleMaterial>);

// This struct defines the data that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SimpleMaterial {

    #[uniform(0)]
    instances_count: u32,

    #[storage(1, read_only)]
    instances_factors: Handle<ShaderStorageBuffer>,
}

impl Material for SimpleMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
