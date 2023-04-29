use std::collections::HashMap;

use crate::{prelude::*, state::game::Camera};
use bevy_ecs::system::Resource;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Track {
    Ambience,
    World,
}

#[derive(Resource)]
pub struct SoundQueue(pub(crate) HashMap<Track, Vec<SoundInfo>>);

impl SoundQueue {
    pub fn push(&mut self, track: Track, snd_info: SoundInfo) {
        self.0
            .get_mut(&track)
            .unwrap_or_else(|| panic!("{}", expect_msg(&track)))
            .push(snd_info);
    }

    pub fn pop(&mut self, track: Track) -> Option<SoundInfo> {
        self.0
            .get_mut(&track)
            .unwrap_or_else(|| panic!("{}", expect_msg(&track)))
            .pop()
    }
}

fn expect_msg(track: &Track) -> String {
    format!("{:?} should have been added already", track)
}

impl Default for SoundQueue {
    fn default() -> Self {
        let map = [(Track::Ambience, vec![]), (Track::World, vec![])].into();
        Self(map)
    }
}

#[derive(Default)]
pub struct SoundInfo {
    pub path: String,
    pub settings: kira::sound::static_sound::StaticSoundSettings,
}

impl SoundInfo {
    /// Returns sound information with settings to give the effect of 3D audio
    pub fn at_position(path: &str, cam: &Camera, pos: Vec2) -> Self {
        let dir = cam.pos - pos;
        let angle = cam.dir.angle_between(dir);

        let mut pan = (angle.sin() / 2. + 0.5) as f64;
        if pan.is_nan() {
            pan = 0.5;
        }

        let dist = cam.pos.distance_squared(pos) as f64;
        let vol = ((1. / dist) * 2.5).min(1.);
        let settings = kira::sound::static_sound::StaticSoundSettings::new()
            .panning(pan)
            .volume(kira::Volume::Amplitude(vol));

        Self {
            path: path.into(),
            settings,
        }
    }
}
