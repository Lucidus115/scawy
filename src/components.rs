use crate::prelude::*;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Transform {
    pub pos: Vec2,
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
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
pub struct Player;

#[derive(Component)]
pub struct Sprite {
    pub height: f32,
    pub color: [u8; 4],
    pub texture: String,
}
