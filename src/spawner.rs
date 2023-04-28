use bevy_ecs::{prelude::Entity, world::World};

use crate::prelude::*;

use components::*;

pub fn spawn_ray(world: &mut World, trans: Transform) -> Entity {
    world
        .spawn((trans, Collider::default(), Movement::with_speed(100.), Ray))
        .id()
}

pub fn spawn_player(world: &mut World, trans: Transform) -> Entity {
    world
        .spawn((
            trans,
            Movement::with_speed(0.2),
            Player,
            Collider::default(),
        ))
        .id()
}

pub fn spawn_monster(world: &mut World, trans: Transform) -> Entity {
    world
        .spawn((
            trans,
            Monster::Rest(FPS * 20), // 20 second rest period
            Movement::with_speed(0.125),
            Navigator::default(),
        ))
        .id()
}
