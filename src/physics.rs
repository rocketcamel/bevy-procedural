use bevy::prelude::*;

use crate::character::{Grounded, Player, Velocity};

const GRAVITY: f32 = -9.81 * 2.;

pub fn physics(
    mut query: Query<(&mut Transform, &mut Velocity, &mut Grounded), With<Player>>,
    time: Res<Time>,
) {
    for (mut transform, mut velocity, mut grounded) in &mut query {
        velocity.linear.y += GRAVITY * time.delta_secs();

        transform.translation += velocity.linear * time.delta_secs();

        if transform.translation.y <= 1.15 {
            transform.translation.y = 1.15;
            velocity.linear.y = 0.;
            grounded.0 = true;
        } else {
            grounded.0 = false;
        };
    }
}
