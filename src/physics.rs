use crate::{map, prelude::*};
use bevy_ecs::prelude::*;

pub fn apply_movement(
    mut move_query: Query<(&mut components::Transform, &mut components::Movement)>,
) {
    for (mut loc, mut movement) in move_query.iter_mut() {
        loc.pos += velocity(movement.velocity(), movement.speed());
        movement.set_velocity(Vec2::ZERO);
    }
}

pub fn detect_collision(
    map: Res<map::Map>,
    mut move_query: Query<(&components::Transform, &mut components::Movement)>,
) {
    // Reduction of collider size to make movement a bit easier
    // const BORDER: f32 = 0.1;

    for (trans, mut movement) in move_query.iter_mut() {
        let new_pos = trans.pos + velocity(movement.velocity(), movement.speed());

        if let Some(tile) = map.get_tile(new_pos.x as u32, new_pos.y as u32) {
            if *tile == map::Tile::Empty {
                continue;
            }
            movement.set_velocity(Vec2::ZERO);
        }
    }
}

pub fn collide(pos_a: Vec2, size_a: Vec2, pos_b: Vec2, size_b: Vec2) -> bool {
    pos_a.x + size_a.x > pos_b.x
        && pos_a.x < pos_b.x + size_b.x
        && pos_a.y + size_a.y > pos_b.y
        && pos_a.y < pos_b.y + size_b.y
}

fn velocity(vel: Vec2, speed: f32) -> Vec2 {
    vel * speed * TIMESTEP * PPU
}
