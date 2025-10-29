use std::fs;
use std::path::Path;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource as KiraAudioSource;

use td::audio::util::{AUDIO_SFX_DIR, load_first_existing};
use td::audio::{AudioAssets, GameAudioPlugin};

fn assets_dir() -> &'static str {
    "assets"
}

fn sfx_path(stem: &str, ext: &str) -> String {
    format!("{}/{}.{}", AUDIO_SFX_DIR, stem, ext)
}

fn make_dummy_file(rel: &str) {
    let full = Path::new(assets_dir()).join(rel);
    if let Some(parent) = full.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&full, b"test");
}

fn remove_dummy_file(rel: &str) {
    let full = Path::new(assets_dir()).join(rel);
    let _ = fs::remove_file(full);
}

#[test]
fn load_first_existing_prefers_wav_then_flac_then_mp3_then_ogg() {
    let stem = "_unit_test_audio_priority";
    // Ensure clean slate
    for ext in ["wav", "flac", "mp3", "ogg"] {
        remove_dummy_file(&sfx_path(stem, ext));
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default());
    app.init_asset::<KiraAudioSource>();
    let asset_server = app.world().get_resource::<AssetServer>().unwrap().clone();

    // Only mp3 present
    make_dummy_file(&sfx_path(stem, "mp3"));
    let h_mp3 = asset_server.load::<KiraAudioSource>(sfx_path(stem, "mp3"));
    let pick = load_first_existing(&asset_server, stem);
    assert_eq!(pick, h_mp3);
    remove_dummy_file(&sfx_path(stem, "mp3"));

    // flac preferred over mp3
    make_dummy_file(&sfx_path(stem, "flac"));
    make_dummy_file(&sfx_path(stem, "mp3"));
    let h_flac = asset_server.load::<KiraAudioSource>(sfx_path(stem, "flac"));
    let pick = load_first_existing(&asset_server, stem);
    assert_eq!(pick, h_flac);
    remove_dummy_file(&sfx_path(stem, "flac"));
    remove_dummy_file(&sfx_path(stem, "mp3"));

    // wav preferred over flac
    make_dummy_file(&sfx_path(stem, "wav"));
    make_dummy_file(&sfx_path(stem, "flac"));
    let h_wav = asset_server.load::<KiraAudioSource>(sfx_path(stem, "wav"));
    let pick = load_first_existing(&asset_server, stem);
    assert_eq!(pick, h_wav);

    // Cleanup
    remove_dummy_file(&sfx_path(stem, "wav"));
    remove_dummy_file(&sfx_path(stem, "flac"));
}

#[test]
fn load_audio_assets_populates_handles() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(bevy_kira_audio::AudioPlugin)
        .add_plugins(GameAudioPlugin);

    // Run startup to load handles
    app.update();

    let assets = app.world().get_resource::<AudioAssets>().unwrap();
    // Ensure that all handles are created (not default-empty).
    // Comparing to default should be false because default handle has id 0.
    let default_handle: Handle<KiraAudioSource> = Handle::default();
    assert_ne!(assets.tower_bow_release, default_handle);
    assert_ne!(assets.tower_crossbow_release, default_handle);
    assert_ne!(assets.wave_start, default_handle);
    assert_ne!(assets.wave_start_boss, default_handle);
    assert_ne!(assets.player_footstep_01, default_handle);
    assert_ne!(assets.tower_place, default_handle);
    assert_ne!(assets.tower_place_invalid, default_handle);
    assert_ne!(assets.tower_upgrade, default_handle);
    assert_ne!(assets.tower_sell, default_handle);
}
