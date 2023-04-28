use std::collections::HashMap;

use crate::{astar, map, prelude::*, state::game::{add_event, Camera}, sound};
use bevy_ecs::prelude::*;
use rand::Rng;

struct ReachedTarget {
    nav_entity: Entity,
    target: Vec2,
}

pub fn add_to_world(schedule: &mut Schedule, world: &mut World) {
    add_event::<ReachedTarget>(world, schedule);
    schedule.add_systems((
        traverse_path,
        navigate,
        monster_rest_countdown,
        monster_wander,
        monster_rest,
        play_monster_sound,
    ));
}

fn traverse_path(
    mut event_writer: EventWriter<ReachedTarget>,
    mut nav_query: Query<(
        Entity,
        &components::Transform,
        &mut components::Movement,
        &mut components::Navigator,
    )>,
    mut current_node: Local<HashMap<Entity, usize>>,
) {
    const MIN_DIST: f32 = 0.2;

    for (ent, trans, mut movement, mut nav) in nav_query.iter_mut() {
        let Some(move_to) = nav.move_to else {
            continue;
        };

        let idx = current_node.entry(ent).or_insert(0);

        // Get closest node position
        if let Some(pos) = nav.path.get(*idx) {
            let pos = *pos + 0.5;
            let dir = pos - trans.pos;

            movement.set_velocity(dir);

            if pos.distance_squared(trans.pos) < MIN_DIST {
                if (pos - 0.5) == move_to {
                    nav.move_to = None;
                    *idx = 0;

                    event_writer.send(ReachedTarget {
                        nav_entity: ent,
                        target: move_to,
                    });

                    continue;
                }
                *idx += 1;
            }
        } else {
            // Chances are the path became shorter and the index is no longer valid
            *idx = 0;
        }
    }
}

fn navigate(
    map: Res<crate::map::Map>,
    mut query: Query<(&components::Transform, &mut components::Navigator)>,
) {
    for (trans, mut nav) in query.iter_mut() {
        if let Some(move_to) = nav.move_to {
            nav.path = astar::navigate(&map, trans.pos, move_to);
        }
    }
}

fn monster_rest_countdown(mut query: Query<&mut components::Monster>) {
    for mut monster in query.iter_mut() {
        let components::Monster::Rest(ticks) = *monster else {
            continue;
        };

        if ticks == 0 {
            *monster = components::Monster::Wander;
            return;
        }
        *monster = components::Monster::Rest(ticks - 1);
    }
}

fn play_monster_sound(
    mut sounds: ResMut<sound::SoundQueue>,
    cam: Res<Camera>,
    query: Query<(&components::Transform, &components::Monster)>,
    mut snd_timer: Local<u32>,
) {
    if *snd_timer != 0 {
        *snd_timer -= 1;
        return;
    }

    for (trans, monster) in query.iter() {
        if let components::Monster::Rest(_) = monster {
            continue;
        }

        let dir = cam.pos - trans.pos;
        let angle = cam.dir.angle_between(dir);

        let mut pan = (angle.sin() / 2. + 0.5) as f64;
        if pan.is_nan() {
            pan = 0.5;
        }

        let dist = cam.pos.distance_squared(trans.pos) as f64;
        let vol = ((1. / dist) * 2.5).min(1.);
        let settings = kira::sound::static_sound::StaticSoundSettings::new()
            .panning(pan)
            .volume(kira::Volume::Amplitude(vol));
        // Attempt to play wander sound
        sounds.push(sound::SoundInfo {
            path: "step.wav".into(),
            settings
        })
    }
    // Play sound every 1.25 seconds
    *snd_timer = (FPS as f32 * 1.25) as u32;
}

fn monster_wander(
    mut query: Query<(&components::Monster, &mut components::Navigator)>,
    map: Res<map::Map>,
) {
    for (monster, mut nav) in query.iter_mut() {
        let components::Monster::Wander = *monster else {
            continue;
        };

        if nav.move_to.is_some() {
            return;
        }

        // Pick a random spot on the map to go to
        let x = rand::thread_rng().gen_range(0..map.width());
        let y = rand::thread_rng().gen_range(0..map.height());

        if map.get_tile(x, y) == Some(&map::Tile::Empty) {
            nav.move_to = Some(vec2(x as f32, y as f32));
        }
    }
}

fn monster_rest(
    mut event_reader: EventReader<ReachedTarget>,
    mut query: Query<&mut components::Monster>,
) {
    for event in event_reader.iter() {
        if let Ok(mut monster) = query.get_mut(event.nav_entity) {
            let rest_time = rand::thread_rng().gen_range(2..6);
            *monster = components::Monster::Rest(rest_time * FPS);
        }
    }
}
