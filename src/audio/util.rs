use bevy::prelude::*;
use bevy_kira_audio::AudioSource as KiraAudioSource;

pub const AUDIO_SFX_DIR: &str = "audio/sfx";
pub const AUDIO_EXTS: [&str; 4] = ["wav", "flac", "mp3", "ogg"];

pub fn load_first_existing(asset_server: &AssetServer, stem: &str) -> Handle<KiraAudioSource> {
    for ext in AUDIO_EXTS {
        let rel = format!("{}/{}.{}", AUDIO_SFX_DIR, stem, ext);
        let full = std::path::Path::new("assets").join(&rel);
        if full.exists() {
            return asset_server.load(rel);
        }
    }
    // Fallback to ogg to produce a stable handle; may fail to load if missing
    asset_server.load(format!("{}/{}.ogg", AUDIO_SFX_DIR, stem))
}
