mod camera;
mod character;
mod physics;

use crate::camera::{CameraPlugin, spawn_camera};
use crate::character::{character_input, spawn_character};
use crate::physics::physics;
use bevy::light::NotShadowCaster;
use bevy::{color::palettes::css::GRAY, prelude::*};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CameraPlugin))
        .add_systems(Startup, (world, spawn_character, spawn_camera).chain())
        .add_systems(Update, (character_input, physics).chain())
        .run();
}

fn world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(10.0, 0.5, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GRAY.into(),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(-2.5, 4.5, 9.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            // base_color: Srgba::hex("888888").unwrap().into(),
            base_color: Srgba::hex("#ADD8E6").unwrap().into(),
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_scale(Vec3::splat(200.)),
        NotShadowCaster,
    ));
}
