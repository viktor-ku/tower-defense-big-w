use crate::audio::util::load_first_existing;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource as KiraAudioSource;

pub const STEM: &str = "wave_start";

pub fn load(asset_server: &AssetServer) -> Handle<KiraAudioSource> {
    load_first_existing(asset_server, STEM)
}
