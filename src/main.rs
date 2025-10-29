#![forbid(unsafe_code)]

use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;
mod constants;

mod audio;
mod build;
mod components;
mod core;
mod entities;
mod events;
mod materials;
mod random_policy;
mod setup;
mod splash;
mod systems;

use build::BuildPlugin;
use components::*;
use constants::Tunables;
use events::*;
use materials::*;
use random_policy::RandomizationPolicy;
use setup::*;
use splash::SplashPlugin;
use systems::camera::camera_system;
use systems::chunks::ChunkPlugin;
use systems::combat::assets::{CombatVfxAssets, init_combat_vfx_assets};
use systems::combat::enemy::{enemy_spawning, face_enemy_health_bars, update_enemy_health_bars};
use systems::combat::projectiles::{
    damage_dealt_spawn_text_system, damage_number_system, enemy_fade_out_system,
    enemy_flash_system, impact_effect_system, projectile_system, tower_shooting,
};
use systems::combat::towers::{
    tower_building, tower_damage_label_spawner, tower_damage_label_system, tower_selling_click,
    tower_spawn_effect_system, update_tower_damage_labels,
};
use systems::input::{handle_game_input, handle_menu_input, pause_toggle_input};
use systems::movement::{enemy_movement, player_movement};
use systems::resource_passes::{
    ResourcePassesPlugin, RocksAlongRoadPassPlugin, TownSquareExclusionPassPlugin,
};
use systems::tree_collection::{
    hold_to_collect, resource_collected_spawn_text_system, resource_number_system,
};
use systems::ui::collect_bar::{CollectUiState, manage_collect_bar_ui};
use systems::ui::hud::{
    spawn_game_speed_indicator, spawn_resource_counters, spawn_village_health_bar, spawn_wave_hud,
    update_currency_counters, update_game_speed_indicator, update_resource_counters,
    update_wave_hud, village_health_hud,
};
use systems::ui::observers::{
    on_enemy_killed, on_enemy_spawned, on_resource_collected, on_tower_built,
};
use systems::ui::warmup::warm_ui_pipelines;
use systems::waves::wave_progression;
use systems::window::force_exit_on_close;
// Frame time graph (Bevy 0.17 dev tools)
#[cfg(feature = "devtools")]
use bevy::dev_tools::frame_time_graph::FrameTimeGraphPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use rand::Rng;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Determine the world seed for this run: allow --seed override, otherwise randomize.
    let launch_seed = determine_launch_seed();

    // Start from default tunables, then inject the dynamic seed before the app/plugins read it.
    let mut tunables = Tunables::default();
    tunables.world_seed = launch_seed;

    // Persist the used seed so we can reproduce a given world later if needed.
    persist_seed_to_app_data(launch_seed);

    let mut app = App::new();
    app.insert_resource(tunables.clone())
        .insert_resource(WaveState::new(&tunables))
        .insert_resource(CombatVfxAssets::default())
        .insert_resource(RandomizationPolicy::default())
        .add_plugins((DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: tunables.window_title.into(),
                    resolution: (tunables.window_resolution.0, tunables.window_resolution.1).into(),
                    ..default()
                }),
                exit_condition: bevy::window::ExitCondition::OnPrimaryClosed,
                close_when_requested: true,
                ..default()
            })
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::INFO,
                filter: "wgpu=error,bevy_render=error".into(),
                ..default()
            }),))
        .add_plugins(bevy_kira_audio::prelude::AudioPlugin)
        .add_plugins(audio::GameAudioPlugin)
        .add_plugins((
            MaterialPlugin::<ProjectileMaterial>::default(),
            MaterialPlugin::<ImpactMaterial>::default(),
        ))
        .add_plugins(ChunkPlugin)
        .add_plugins(ResourcePassesPlugin)
        .add_plugins(RocksAlongRoadPassPlugin)
        .add_plugins(TownSquareExclusionPassPlugin)
        .add_plugins(SplashPlugin)
        .add_plugins(BuildPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default());

    // Dev tools (frame time graph) only in devtools feature
    #[cfg(feature = "devtools")]
    {
        app.add_plugins(FrameTimeGraphPlugin);
    }

    // Add explicit exit handling and the rest of the systems/plugins
    app.add_systems(Update, bevy::window::close_when_requested)
        .add_systems(Update, bevy::window::exit_on_all_closed)
        .init_state::<GameState>()
        .insert_state(GameState::Loading)
        .insert_resource(CurrentCollectProgress::default())
        .insert_resource(CollectUiState::default())
        .insert_resource(TowerBuildSelection::default())
        .add_message::<ResourceCollected>()
        .add_message::<TowerBuilt>()
        .add_message::<EnemySpawned>()
        .add_message::<EnemyKilled>()
        .add_message::<DamageDealt>()
        .add_message::<bevy::window::WindowCloseRequested>()
        .add_message::<AppExit>()
        .add_systems(
            OnEnter(GameState::Loading),
            (
                setup,
                init_combat_vfx_assets,
                warm_ui_pipelines,
                spawn_village_health_bar,
                spawn_resource_counters,
                spawn_wave_hud,
                spawn_game_speed_indicator,
            ),
        )
        .add_systems(Update, handle_menu_input.run_if(in_state(GameState::Menu)))
        .add_systems(
            Update,
            handle_game_input.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            pause_toggle_input.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            pause_toggle_input.run_if(in_state(GameState::Paused)),
        )
        .add_systems(Update, player_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, tower_building.run_if(in_state(GameState::Playing)))
        .add_systems(
            Update,
            tower_damage_label_spawner.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            tower_selling_click.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            wave_progression.run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, enemy_spawning.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, tower_shooting.run_if(in_state(GameState::Playing)))
        .add_systems(
            Update,
            tower_spawn_effect_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                projectile_system,
                damage_dealt_spawn_text_system,
                enemy_fade_out_system,
                impact_effect_system,
                enemy_flash_system,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Position floating damage numbers after transforms have propagated
        .add_systems(
            PostUpdate,
            (
                damage_number_system.after(camera_system),
                tower_damage_label_system.after(camera_system),
                update_tower_damage_labels,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Observers for gameplay events (logging)
        .add_observer(on_resource_collected)
        .add_observer(on_tower_built)
        .add_observer(on_enemy_spawned)
        .add_observer(on_enemy_killed)
        // Camera system: run after transform propagation so it sees latest positions
        .add_systems(
            PostUpdate,
            camera_system.run_if(in_state(GameState::Playing)),
        )
        // HUD systems
        .add_systems(
            Update,
            (
                village_health_hud,
                update_resource_counters,
                update_currency_counters,
                update_wave_hud,
                manage_collect_bar_ui,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Game speed indicator updates every frame to also hide in non-game states
        .add_systems(Update, update_game_speed_indicator)
        .add_systems(
            Update,
            (
                face_enemy_health_bars.run_if(bevy::time::common_conditions::on_timer(
                    std::time::Duration::from_millis(33),
                )),
                update_enemy_health_bars,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Tree collection system
        .add_systems(Update, hold_to_collect.run_if(in_state(GameState::Playing)))
        // Resource collection number systems
        .add_systems(
            Update,
            resource_collected_spawn_text_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            PostUpdate,
            resource_number_system
                .after(camera_system)
                .run_if(in_state(GameState::Playing)),
        )
        // Window close handling - use force exit for immediate termination
        .add_systems(Update, force_exit_on_close);

    app.run();
}

// Default font is specified per-usage in UI systems to ensure explicit control.

/// Parse command-line arguments for an explicit seed, otherwise generate a random one.
fn determine_launch_seed() -> u64 {
    // Accept either --seed=NUMBER or --seed NUMBER
    let mut args = std::env::args().skip(1);
    let mut pending_seed_flag = false;
    while let Some(arg) = args.next() {
        if pending_seed_flag {
            if let Ok(value) = arg.parse::<u64>() {
                return value;
            }
            // If malformed, ignore and continue to random seed
            pending_seed_flag = false;
            continue;
        }

        if let Some(rest) = arg.strip_prefix("--seed=") {
            if let Ok(value) = rest.parse::<u64>() {
                return value;
            }
        } else if arg == "--seed" {
            pending_seed_flag = true;
        }
    }

    // No explicit seed provided: generate a random 64-bit seed
    let seed: u64 = rand::rng().random();
    println!("[td] Launching with random world seed: {}", seed);
    seed
}

/// Save the seed into the platform-specific app data directory under td/seed.txt.
fn persist_seed_to_app_data(seed: u64) {
    // Prefer a standard data dir; fall back to current dir if unavailable.
    let base_dir: PathBuf = match dirs_next::data_dir() {
        Some(p) => p,
        None => match std::env::current_dir() {
            Ok(p) => p,
            Err(_) => return, // Give up quietly if we can't determine any directory
        },
    };

    let dir = base_dir.join("td");
    let file_path = dir.join("seed.txt");

    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!(
            "[td] Warning: failed to create app data directory at {:?}: {}",
            dir, e
        );
        return;
    }

    // Write the seed as plain text
    match fs::File::create(&file_path) {
        Ok(mut f) => {
            if let Err(e) = writeln!(f, "{}", seed) {
                eprintln!(
                    "[td] Warning: failed to write seed to {:?}: {}",
                    file_path, e
                );
            } else {
                println!("[td] Saved world seed {} to {:?}", seed, file_path);
            }
        }
        Err(e) => {
            eprintln!(
                "[td] Warning: failed to open seed file {:?}: {}",
                file_path, e
            );
        }
    }
}
