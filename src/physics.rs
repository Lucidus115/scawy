use crate::{map, prelude::*, state::game::add_event};
use bevy_ecs::prelude::*;

pub struct CollisionHit {
    pub entity: Entity,
    pub hit_entity: Entity,
}

impl CollisionHit {
    pub fn contains(&self, ent: Entity) -> bool {
        self.entity == ent || self.hit_entity == ent
    }
}

pub fn add_to_world(schedule: &mut Schedule, world: &mut World) {
    add_event::<CollisionHit>(world, schedule);
    schedule.add_systems((
        apply_movement,
        detect_collision.before(apply_movement),
        step_ray,
        despawn_ray_on_hit
    ));
}

fn apply_movement(mut move_query: Query<(&mut components::Transform, &mut components::Movement)>) {
    for (mut loc, mut movement) in move_query.iter_mut() {
        loc.pos += velocity(movement.velocity(), movement.speed());
        movement.set_velocity(Vec2::ZERO);
    }
}

fn detect_collision(
    mut event_writer: EventWriter<CollisionHit>,
    map: Res<map::Map>,
    mut move_query: Query<(Entity, &mut components::Movement)>,
    collider_query: Query<(Entity, &components::Transform, &components::Collider)>,
) {
    for (ent_a, mut movement) in move_query.iter_mut() {
        let Ok((_, trans_a, col_a)) = collider_query.get(ent_a) else {
            continue;
        };

        let new_pos = trans_a.pos + velocity(movement.velocity(), movement.speed());

        if let Some(tile) = map.get_tile(new_pos.x as u32, new_pos.y as u32) {
            if *tile == map::Tile::Empty {
                continue;
            }
            let event = CollisionHit {
                entity: ent_a,
                // Use placeholder if hit object is a tile
                // because I don't feel like wrapping in an option
                hit_entity: Entity::PLACEHOLDER,
            };
            event_writer.send(event);
            movement.set_velocity(Vec2::ZERO);
        }

        for (ent_b, trans_other, col_other) in collider_query.iter() {
            if collide(trans_a.pos, col_a.size, trans_other.pos, col_other.size) {
                let event = CollisionHit {
                    entity: ent_a,
                    hit_entity: ent_b,
                };
                event_writer.send(event);
            }
        }
    }
}

fn despawn_ray_on_hit(
    mut cmd: Commands,
    mut event_reader: EventReader<CollisionHit>,
    query: Query<&components::Ray>
) {
    for event in event_reader.iter() {
        let (ent, ray) = if let Ok(ray) = query.get(event.entity) {
            (event.entity, ray)
        } else if let Ok(ray) = query.get(event.hit_entity) {
            (event.hit_entity, ray)
        } else {
            continue;
        };

        if event.contains(ray.parent) {
            continue;
        }
        cmd.entity(ent).despawn();
    }
}

fn step_ray(mut query: Query<(&mut components::Movement, &components::Transform), With<components::Ray>>) {
    for (mut movement, trans) in query.iter_mut() {
        movement.set_velocity(trans.dir);
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
