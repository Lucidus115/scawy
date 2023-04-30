use std::collections::HashMap;

use crate::{
    astar,
    components::{Monster, MonsterState},
    map,
    prelude::*,
    sound,
    state::game::{add_event, Camera},
    ticks,
};
use bevy_ecs::prelude::*;
use rand::Rng;

pub const ATTACK_TIME: u32 = (FPS as f32 * 2.5) as u32;

const SEEK_TIME: f32 = 10.;
const ATTACK_RANGE: f32 = 1.;
const AGGRO_SPEED_MULTIPLIER: f32 = 1.15;

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
        set_target,
        attack,
        flee,
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

fn monster_rest_countdown(mut query: Query<&mut Monster>) {
    for mut monster in query.iter_mut() {
        let MonsterState::Rest(ticks) = monster.state else {
            continue;
        };

        if ticks == 0 {
            monster.state = MonsterState::Wander;
            return;
        }
        monster.state = MonsterState::Rest(ticks - 1);
        monster.attack_time = ATTACK_TIME;
    }
}

fn play_monster_sound(
    mut sounds: ResMut<sound::SoundQueue>,
    cam: Res<Camera>,
    query: Query<(&components::Transform, &Monster)>,
    mut snd_timer: Local<u32>,
) {
    if *snd_timer != 0 {
        *snd_timer -= 1;
        return;
    }

    let mut seconds_to_play = 1.25;

    for (trans, monster) in query.iter() {
        match monster.state {
            MonsterState::Rest(_) => continue,
            MonsterState::Flee(_) | MonsterState::Attack(_) => {
                seconds_to_play = 0.5;
            }
            _ => (),
        }

        // Attempt to play wander sound
        sounds.push(
            sound::Track::Sfx,
            sound::SoundInfo::at_position("step.wav", &cam, trans.pos),
        )
    }
    // Play sound every 1.25 seconds
    *snd_timer = ticks(seconds_to_play);
}

fn monster_wander(mut query: Query<(&Monster, &mut components::Navigator)>, map: Res<map::Map>) {
    for (monster, mut nav) in query.iter_mut() {
        let MonsterState::Wander = monster.state else {
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

fn monster_rest(mut event_reader: EventReader<ReachedTarget>, mut query: Query<&mut Monster>) {
    for event in event_reader.iter() {
        if let Ok(mut monster) = query.get_mut(event.nav_entity) {
            let rest_time = rand::thread_rng().gen_range(2..6);
            monster.state = MonsterState::Rest(ticks(rest_time as f32));
        }
    }
}

fn set_target(
    mut query: Query<(&mut Monster, &mut components::Movement)>,
    target_query: Query<Entity, With<components::MonsterTarget>>,
    mut timer: Local<u32>,
) {
    if *timer != 0 {
        *timer -= 1;
        return;
    }

    let targets: Vec<Entity> = target_query.iter().collect();
    for (mut monster, mut movement) in query.iter_mut() {
        let MonsterState::Wander = monster.state else {
            continue;
        };
        if !rand::thread_rng().gen_bool(1. / 6.) {
            continue;
        }

        if let Some(target) = targets.get(rand::thread_rng().gen_range(0..targets.len())) {
            monster.state = MonsterState::Attack(*target);
        }

        let speed = movement.speed() * AGGRO_SPEED_MULTIPLIER;
        movement.set_speed(speed);
    }

    *timer = ticks(SEEK_TIME);
}

fn attack(
    mut sounds: ResMut<sound::SoundQueue>,
    mut query: Query<(
        &components::Transform,
        &mut Monster,
        &mut components::Movement,
        &mut components::Navigator,
    )>,
    mut target_query: Query<
        (
            &components::Transform,
            &mut components::MonsterTarget,
            Option<&mut components::Movement>,
        ),
        Without<components::Monster>,
    >,
) {
    'outer: for (trans, mut monster, mut movement, mut nav) in query.iter_mut() {
        match monster.state {
            MonsterState::Flee(_) | MonsterState::Rest(_) => continue,
            _ => (),
        }

        for (target_trans, mut target, movement) in target_query.iter_mut() {
            if trans.pos.distance_squared(target_trans.pos) < ATTACK_RANGE {
                if monster.attack_time != 0 {
                    if let Some(mut movement) = movement {
                        movement.set_velocity(Vec2::ZERO);
                    }

                    // Really stupid but I'm too lazy to add another timer
                    if monster.attack_time == ATTACK_TIME {
                        sounds.push(
                            sound::Track::Sfx,
                            sound::SoundInfo {
                                path: "hurt.wav".into(),
                                ..Default::default()
                            },
                        );
                    }

                    monster.attack_time -= 1;
                    continue 'outer;
                }
                target.is_dead = true;
                monster.attack_time = ATTACK_TIME;
            }
        }

        let MonsterState::Attack(ent) = monster.state else {
            continue;
        };

        let Ok((target_trans, target, _)) = target_query.get(ent) else {
            continue;
        };

        nav.move_to = Some(target_trans.pos);

        if target.is_dead {
            monster.state = MonsterState::Rest(ticks(15.));

            let speed = movement.speed() / AGGRO_SPEED_MULTIPLIER;
            movement.set_speed(speed);
        }
    }
}

fn flee(mut query: Query<(&mut components::Navigator, &components::Monster)>) {
    for (mut nav, monster) in query.iter_mut() {
        let MonsterState::Flee(pos) = monster.state else {
            continue;
        };
        nav.move_to = Some(pos);
    }
}
