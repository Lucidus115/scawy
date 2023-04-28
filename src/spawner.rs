use bevy_ecs::{prelude::Entity, system::Commands};

use crate::prelude::*;

use components::*;

pub fn spawn_ray(cmd: &mut Commands, trans: Transform, ray: Ray) -> Entity {
    cmd.spawn((
        trans,
        Collider::default(),
        Movement::with_speed(100.),
        ray,
    ))
    .id()
}

pub fn spawn_player(cmd: &mut Commands, trans: Transform) -> Entity {
    cmd.spawn((
        trans,
        Movement::with_speed(0.2),
        Player,
        Collider::default(),
    ))
    .id()
}

pub fn spawn_monster(cmd: &mut Commands, trans: Transform) -> Entity {
    cmd.spawn((
        trans,
        Monster::Rest(FPS * 20), // 20 second rest period
        Movement::with_speed(0.125),
        Navigator::default(),
    ))
    .id()
}
