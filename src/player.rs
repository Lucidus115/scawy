use std::collections::HashMap;

use crate::{
    map::Map,
    prelude::*,
    sound,
    state::game::{add_event, Camera, CoreSet, GameData},
    ticks,
};
use bevy_ecs::prelude::*;
use rand::Rng;

const LIGHT_RANGE: f32 = 16.;
const INTERACT_RANGE: f32 = 1.5;

pub enum Action {
    Interact,
    Attack,
}

pub struct SendAction {
    pub entity: Entity,
    pub action: Action,
}

pub struct FlashLight {
    pub intesity: f32,
    pub duration: u32,
}

pub enum ExitCondition {
    Win,
    Lose,
}

struct Interact {
    entity: Entity,
}

pub fn add_to_world(schedule: &mut Schedule, world: &mut World) {
    add_event::<SendAction>(world, schedule);
    add_event::<FlashLight>(world, schedule);
    add_event::<ExitCondition>(world, schedule);
    add_event::<Interact>(world, schedule);

    schedule.add_systems((
        cam_follow_player,
        turn_on_gen,
        use_light,
        pickup_battery,
        play_gen_sound,
        exit_door,
        exit_on_dead,
        interact.in_base_set(CoreSet::First),
        despawn_interactable.in_base_set(CoreSet::Last),
    ));
}

fn interact(
    mut event_reader: EventReader<SendAction>,
    mut event_writer: EventWriter<Interact>,
    player_query: Query<&components::Transform, With<components::Player>>,
    interactable_query: Query<(Entity, &components::Transform), With<components::Interactable>>,
) {
    for event in event_reader.iter() {
        let Action::Interact = event.action else {
            continue;
        };
        for player_trans in player_query.iter() {
            for (int, int_trans) in interactable_query.iter() {
                if player_trans.pos.distance_squared(int_trans.pos) < INTERACT_RANGE {
                    event_writer.send(Interact { entity: int })
                }
            }
        }
    }
}
fn cam_follow_player(
    mut cam: ResMut<Camera>,
    query: Query<&components::Transform, With<components::Player>>,
) {
    for trans in query.iter() {
        cam.pos = trans.pos;
    }
}

fn use_light(
    data: Res<GameData>,
    map: Res<Map>,
    mut sounds: ResMut<sound::SoundQueue>,
    mut event_writer: EventWriter<FlashLight>,
    mut event_reader: EventReader<SendAction>,
    mut query: Query<(&components::Transform, &mut components::Player)>,
    mut monster_query: Query<(&components::Transform, &mut components::Monster)>,
) {
    for event in event_reader.iter() {
        if data.generators_required == 0 {
            continue;
        }
        let Action::Attack = event.action else {
            continue;
        };
        let Ok((player_trans, mut player)) = query.get_mut(event.entity) else {
            continue;
        };

        if player.batteries == 0 {
            sounds.push(
                sound::Track::Sfx,
                sound::SoundInfo {
                    path: "click.wav".into(),
                    ..Default::default()
                },
            );
            return;
        }
        player.batteries -= 1;

        for (monster_trans, mut monster) in monster_query.iter_mut() {
            if player_trans.pos.distance_squared(monster_trans.pos) > LIGHT_RANGE {
                continue;
            }

            // Find a random point on map to flee to
            let mut pos = monster_trans.pos;

            while pos.distance_squared(player_trans.pos) < 100. {
                let idx = rand::thread_rng().gen_range(0..map.width() * map.height());
                let x = idx % map.width();
                let y = idx / map.height();

                if map.get_tile(x, y) == Some(&crate::map::Tile::Empty) {
                    pos = vec2(x as f32, y as f32);
                }
            }
            monster.state = components::MonsterState::Flee(pos);
        }

        event_writer.send(FlashLight {
            intesity: 7.,
            duration: (FPS as f32 * 0.5) as u32,
        });

        sounds.push(
            sound::Track::Sfx,
            sound::SoundInfo {
                path: "flash.wav".into(),
                ..Default::default()
            },
        );
    }
}

fn play_gen_sound(
    mut sounds: ResMut<sound::SoundQueue>,
    cam: Res<Camera>,
    gen_query: Query<(Entity, &components::Transform, &components::Generator)>,
    mut sound_ticks: Local<HashMap<Entity, u32>>,
) {
    let ticks_til_start = ticks(1.75);

    for (ent, trans, gen) in gen_query.iter() {
        if !gen.is_on {
            continue;
        }

        let Some(start) = sound_ticks.get_mut(&ent) else {
            sound_ticks.insert(ent, ticks_til_start);
            continue;
        };

        if *start != 0 {
            *start -= 1;
            continue;
        }

        let mut snd = sound::SoundInfo::at_position("generator_running.wav", &cam, trans.pos);
        let vol = snd.settings.volume.as_amplitude();
        snd.settings.volume = kira::Volume::Amplitude(vol * 0.15);
        sounds.push(sound::Track::Sfx, snd);
    }
}

fn turn_on_gen(
    mut int_reader: EventReader<Interact>,
    mut light_writer: EventWriter<FlashLight>,
    mut data: ResMut<GameData>,
    mut sounds: ResMut<sound::SoundQueue>,
    cam: Res<Camera>,
    mut gen_query: Query<(&components::Transform, &mut components::Generator)>,
) {
    for event in int_reader.iter() {
        let Ok((trans, mut gen)) = gen_query.get_mut(event.entity) else {
            continue;
        };
        if gen.is_on {
            continue;
        }

        let snd = sound::SoundInfo::at_position("generator_on.wav", &cam, trans.pos);
        sounds.push(sound::Track::Sfx, snd);

        gen.is_on = true;
        data.generators_required -= 1;

        if data.generators_required == 0 {
            light_writer.send(FlashLight {
                intesity: f32::MAX,
                duration: u32::MAX,
            });

            let snd = sound::SoundInfo {
                path: "power_on.wav".into(),
                ..Default::default()
            };

            sounds.push(sound::Track::Sfx, snd);
        }
    }
}

fn pickup_battery(
    mut int_reader: EventReader<Interact>,
    mut player_query: Query<&mut components::Player>,
    bat_query: Query<&components::Battery>,
) {
    for event in int_reader.iter() {
        for mut player in player_query.iter_mut() {
            let Ok(bat) = bat_query.get(event.entity) else {
            continue;
        };

            player.batteries += bat.amount;
        }
    }
}

fn despawn_interactable(
    mut cmd: Commands,
    mut int_reader: EventReader<Interact>,
    int_query: Query<&components::Interactable, Without<components::Generator>>,
) {
    for event in int_reader.iter() {
        if int_query.get(event.entity).is_ok() {
            cmd.entity(event.entity).despawn();
        }
    }
}

fn exit_door(
    mut event_writer: EventWriter<ExitCondition>,
    data: Res<GameData>,
    query: Query<&components::Transform, With<components::Player>>,
    exit_query: Query<&components::Transform, With<components::Exit>>,
) {
    for trans_a in query.iter() {
        if data.generators_required != 0 {
            continue;
        }

        // power is on check if next to an exit
        for trans_b in exit_query.iter() {
            if trans_a.pos.distance_squared(trans_b.pos) > 1. {
                continue;
            }
            event_writer.send(ExitCondition::Win)
        }
    }
}

fn exit_on_dead(
    mut event_writer: EventWriter<ExitCondition>,
    query: Query<&components::MonsterTarget, With<components::Player>>,
) {
    for target in query.iter() {
        if target.is_dead {
            event_writer.send(ExitCondition::Lose);
        }
    }
}
