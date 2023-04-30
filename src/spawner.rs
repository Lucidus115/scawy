use bevy_ecs::{prelude::Entity, system::Commands};

use crate::{ai, prelude::*, ticks};

use components::*;

pub fn spawn_player(cmd: &mut Commands, trans: Transform) -> Entity {
    cmd.spawn((
        trans,
        Movement::with_speed(0.2),
        Player::default(),
        Collider::default(),
        MonsterTarget::default(),
    ))
    .id()
}

pub fn spawn_monster(cmd: &mut Commands, trans: Transform) -> Entity {
    cmd.spawn((
        trans,
        Monster {
            state: MonsterState::Rest(ticks(20.)),
            attack_time: ai::ATTACK_TIME,
        },
        Movement::with_speed(0.125),
        Navigator::default(),
    ))
    .id()
}
