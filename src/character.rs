use bevy::{color::palettes::css::GRAY, math::VectorSpace, prelude::*};

use crate::camera::PanOrbitState;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component)]
pub struct Grounded(pub bool);

#[derive(Component)]
pub struct Velocity {
    pub linear: Vec3,
}

impl Default for Velocity {
    fn default() -> Self {
        Self { linear: Vec3::ZERO }
    }
}

const PLAYER_SPEED: f32 = 5.0;
const JUMP_FORCE: f32 = 10.0;

pub fn spawn_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Player,
        LocalPlayer,
        Velocity::default(),
        Grounded(false),
        Mesh3d(meshes.add(Capsule3d::new(0.4, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GRAY.into(),
            ..Default::default()
        })),
        Transform::from_xyz(0., 2.0, 0.),
    ));
}

pub fn character_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &Grounded), With<Player>>,
    camera_query: Single<&Transform, With<PanOrbitState>>,
) {
    let camera_transform = camera_query.into_inner();
    let camera_forward = camera_transform.forward();
    let camera_right = camera_transform.right();

    let forward = Vec3::new(camera_forward.x, 0., camera_forward.z).normalize();
    let right = Vec3::new(camera_right.x, 0., camera_right.z).normalize();

    for (mut velocity, grounded) in &mut query {
        let mut movement = Vec3::ZERO;

        for key in keyboard_input.get_pressed() {
            match key {
                KeyCode::KeyW => movement += forward,
                KeyCode::KeyA => movement -= right,
                KeyCode::KeyD => movement += right,
                KeyCode::KeyS => movement -= forward,
                KeyCode::Space => {
                    if grounded.0 {
                        velocity.linear.y = JUMP_FORCE
                    }
                }
                _ => {}
            }
        }

        if movement.length() > 0.0 {
            movement = movement * PLAYER_SPEED;
            velocity.linear.x = movement.x;
            velocity.linear.z = movement.z;
        }
    }
}
