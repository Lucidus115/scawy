use crate::{graphics::Color, prelude::*};
use bevy_ecs::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Transform {
    pub pos: Vec2,
    pub dir: Vec2,
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            dir: Vec2::NEG_X,
            scale: Vec2::splat(1.),
        }
    }
}

#[derive(Component)]
pub struct Movement {
    vel: Vec2,
    speed: f32,
}

impl Movement {
    pub fn with_speed(speed: f32) -> Self {
        Self {
            vel: Vec2::ZERO,
            speed,
        }
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn velocity(&self) -> Vec2 {
        self.vel
    }

    pub fn set_velocity(&mut self, vel: Vec2) {
        self.vel = vel.normalize_or_zero();
    }
}

#[derive(Component)]
pub struct Collider {
    pub size: Vec2,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            size: Vec2::splat(1.),
        }
    }
}

#[derive(Component, Default)]
pub struct Player {
    pub batteries: u32,
}

#[derive(Component)]
pub struct Ray {
    // The entity which this ray came from
    pub parent: Entity,
    pub max_dist: f32,
}

#[derive(Component)]
pub enum Monster {
    Rest(u32), // Duration to rest for in game ticks
    Wander,
    Attack,
}

#[derive(Component, Default)]
pub struct Navigator {
    pub move_to: Option<Vec2>,
    pub path: Vec<Vec2>,
}

#[derive(Component, Default)]
pub struct Sprite {
    pub height: f32,
    pub color: Color,
    pub texture: String,
}

#[derive(Component, Default)]
pub struct Generator {
    pub is_on: bool,
}

#[derive(Component, Default)]
pub struct Battery {
    pub amount: u32,
}
