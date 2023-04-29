use std::collections::HashMap;

use crate::{
    prelude::*,
    sound, spawner,
    state::game::{add_event, Camera, GameData},
    ticks,
};
use bevy_ecs::prelude::*;

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

pub fn add_to_world(schedule: &mut Schedule, world: &mut World) {
    add_event::<SendAction>(world, schedule);
    add_event::<FlashLight>(world, schedule);
    schedule.add_systems((
        cam_follow_player,
        interact,
        turn_on_gen,
        use_light,
        pickup_battery,
        play_gen_sound,
    ));
}

fn cam_follow_player(
    mut cam: ResMut<Camera>,
    query: Query<&components::Transform, With<components::Player>>,
) {
    for trans in query.iter() {
        cam.pos = trans.pos;
    }
}

fn interact(
    mut cmd: Commands,
    mut event_reader: EventReader<SendAction>,
    query: Query<(Entity, &components::Transform), With<components::Player>>,
) {
    for event in event_reader.iter() {
        let Action::Interact = event.action else {
            continue;
        };
        let Ok((ent, trans)) = query.get(event.entity) else {
            continue;
        };

        spawner::spawn_ray(
            &mut cmd,
            *trans,
            components::Ray {
                parent: ent,
                max_dist: 1.,
            },
        );
    }
}

fn use_light(
    mut sounds: ResMut<sound::SoundQueue>,
    mut event_writer: EventWriter<FlashLight>,
    mut event_reader: EventReader<SendAction>,
    mut query: Query<&mut components::Player>,
) {
    for event in event_reader.iter() {
        let Action::Attack = event.action else {
            continue;
        };
        let Ok(mut player) = query.get_mut(event.entity) else {
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

#[allow(clippy::too_many_arguments)]
fn turn_on_gen(
    mut light_writer: EventWriter<FlashLight>,
    mut collision_reader: EventReader<physics::CollisionHit>,
    mut data: ResMut<GameData>,
    mut sounds: ResMut<sound::SoundQueue>,
    cam: Res<Camera>,
    mut gen_query: Query<(&components::Transform, &mut components::Generator)>,
    ray_query: Query<&components::Ray>,
    player_query: Query<&components::Player>,
) {
    for event in collision_reader.iter() {
        let (par, hit) = if let Ok(ray) = ray_query.get(event.entity) {
            (ray.parent, event.hit_entity)
        } else if let Ok(ray) = ray_query.get(event.hit_entity) {
            (ray.parent, event.entity)
        } else {
            continue;
        };

        if player_query.get(par).is_err() {
            continue;
        }
        let Ok((trans, mut gen)) = gen_query.get_mut(hit) else {
            continue;
        };
        if gen.is_on {
            continue;
        }

        let snd = sound::SoundInfo::at_position("generator_on.wav", &cam, trans.pos);

        snd.settings
            .loop_behavior(kira::LoopBehavior { start_position: 0. });

        sounds.push(sound::Track::Sfx, snd);
        gen.is_on = true;

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
        data.generators_required -= 1;
    }
}

fn pickup_battery(
    mut cmd: Commands,
    mut event_reader: EventReader<physics::CollisionHit>,
    mut player_query: Query<&mut components::Player>,
    bat_query: Query<&components::Battery>,
    ray_query: Query<&components::Ray>,
) {
    for event in event_reader.iter() {
        let (par, hit) = if let Ok(ray) = ray_query.get(event.entity) {
            (ray.parent, event.hit_entity)
        } else if let Ok(ray) = ray_query.get(event.hit_entity) {
            (ray.parent, event.entity)
        } else {
            continue;
        };

        let Ok(mut player) = player_query.get_mut(par) else {
            continue;
        };
        let Ok(bat) = bat_query.get(hit) else {
            continue;
        };

        player.batteries += bat.amount;
        cmd.entity(hit).despawn();
    }
}
