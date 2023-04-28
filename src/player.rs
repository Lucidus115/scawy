use bevy_ecs::prelude::*;
use crate::{prelude::*, state::game::Camera};

pub fn add_to_world(schedule: &mut Schedule, world: &mut World) {
    schedule.add_systems((
        cam_follow_player,
        set_player_direction
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

fn set_player_direction(
    cam: Res<Camera>,
    mut query: Query<&mut components::Transform, With<components::Player>>
) {
    for mut trans in query.iter_mut() {
        trans.dir = cam.dir;
    }
}