extern crate coppervox_logic;
use bevy::color::palettes::css;
use bevy::prelude::*;
use coppervox_logic::logic::{operation_gate::OperationGateBundle, wire::WireState};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Circuit Test".to_string(),
                resolution: (800.0, 600.0).into(),
                present_mode: bevy::window::PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, build_circuit)
        //.add_systems(Update, ( ,))
        .run();
}

type Channel = u32;
type Value = u32;

#[derive(Component, Clone)]
struct MaterialPair {
    material_off: Handle<StandardMaterial>,
    material_on: Handle<StandardMaterial>,
}

#[derive(Component)]
struct MyCamera {}

/// Build logical circuit
fn build_circuit(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // create a simple sphere
    let mesh = meshes.add(Sphere::default());

    // create materials for wires
    let materials_wire = [
        // wire channel 1
        MaterialPair {
            material_off: materials.add(StandardMaterial {
                base_color: css::RED.into(),
                ..default()
            }),
            material_on: materials.add(StandardMaterial {
                base_color: css::RED.into(),
                emissive: css::ORANGE_RED.into(),
                ..default()
            }),
        },
        // wire channel 2
        MaterialPair {
            material_off: materials.add(StandardMaterial {
                base_color: css::GREEN.into(),
                ..default()
            }),
            material_on: materials.add(StandardMaterial {
                base_color: css::GREEN.into(),
                emissive: css::LIMEGREEN.into(),
                ..default()
            }),
        },
        // wire channel 3
        MaterialPair {
            material_off: materials.add(StandardMaterial {
                base_color: css::BLUE.into(),
                ..default()
            }),
            material_on: materials.add(StandardMaterial {
                base_color: css::BLUE.into(),
                emissive: css::ALICE_BLUE.into(),
                ..default()
            }),
        },
    ];

    // create three wires
    let wires = [
        commands
            .spawn((
                WireState::<Channel, Value>::new(0),
                materials_wire[0].clone(),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(materials_wire[0].material_off.clone()),
                Transform::from_xyz(-2.0, 0.0, -20.0),
            ))
            .id(),
        commands
            .spawn((
                WireState::<Channel, Value>::new(1),
                materials_wire[1].clone(),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(materials_wire[1].material_off.clone()),
                Transform::from_xyz(0.0, 0.0, -20.0),
            ))
            .id(),
        commands
            .spawn((
                WireState::<Channel, Value>::new(2),
                materials_wire[2].clone(),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(materials_wire[2].material_off.clone()),
                Transform::from_xyz(2.0, 0.0, -20.0),
            ))
            .id(),
    ];

    commands.spawn(OperationGateBundle::<u32, u32>::new(
        1,
        &[wires[0], wires[1]],
        &[wires[2]],
    ));

    commands.spawn((Camera3d::default(), MyCamera {}));
}
