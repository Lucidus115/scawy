use crate::prelude::*;
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
    pub settings: StaticSoundSettings
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
