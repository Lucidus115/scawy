use crate::{prelude::*, state::game::{Camera, add_event}, spawner};
use bevy_ecs::prelude::*;

pub enum Action {
    Interact,
    Attack,
}

pub struct SendAction {
    pub entity: Entity,
    pub action: Action,
}

pub fn add_to_world(schedule: &mut Schedule, world: &mut World) {
    add_event::<SendAction>(world, schedule);
    schedule.add_systems((cam_follow_player, interact));
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
    query: Query<(Entity, &components::Transform), With<components::Player>>
) {
    for event in event_reader.iter() {
        let Ok((ent, trans)) = query.get(event.entity) else {
            continue;
        };

        spawner::spawn_ray(&mut cmd, *trans, ent);
    }    
}
