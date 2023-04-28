use crate::{prelude::*, state::game::Camera};
use bevy_ecs::system::Resource;
use kira::{
    manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    tween::Tween,
};

use crate::ASSETS_FOLDER;

#[derive(Resource, Default)]
pub struct SoundQueue(pub(crate) Vec<SoundInfo>);

impl SoundQueue {
    pub fn push(&mut self, snd_info: SoundInfo) {
        self.0.push(snd_info);
    }

    pub fn pop(&mut self) -> Option<SoundInfo> {
        self.0.pop()
    }
}

pub struct SoundInfo {
    pub path: String,
    pub settings: StaticSoundSettings,
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

pub struct SoundPlayer {
    manager: AudioManager,
}

impl SoundPlayer {
    pub fn play(&mut self, sound_path: &str, settings: StaticSoundSettings) {
        let path = format!("{}/{sound_path}", ASSETS_FOLDER);
        let Ok(snd) = StaticSoundData::from_file(path, settings) else {
            warn!("Failed to play sound from path: {sound_path}. Path does not exist");
            return;
        };
        if self.manager.play(snd).is_err() {
            warn!("An error occured attempting to play sound from path: {sound_path}");
        }
    }

    pub fn pause(&self) {
        if self.manager.pause(Tween::default()).is_err() {
            warn!("Error occured while pausing the sound player");
        }
    }
}

impl Default for SoundPlayer {
    fn default() -> Self {
        let manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())
            .expect("failed to init Audio Manager");
        Self { manager }
    }
}
