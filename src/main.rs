use bevy::prelude::*;
mod constants;

mod components;
mod events;
mod setup;
mod systems;

use components::*;
use constants::Tunables;
use events::*;
use setup::*;
use systems::*;
// Frame time graph (Bevy 0.17 dev tools)
use bevy::dev_tools::frame_time_graph::FrameTimeGraphPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

fn main() {
    let tunables = Tunables::default();
    App::new()
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
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(FrameTimeGraphPlugin)
        // Add explicit exit handling
        .add_systems(Update, bevy::window::close_when_requested)
        .add_systems(Update, bevy::window::exit_on_all_closed)
        .init_state::<GameState>()
        .insert_state(GameState::Playing)
        .insert_resource(tunables.clone())
        .insert_resource(CurrentCollectProgress::default())
        .insert_resource(CollectUiState::default())
        .add_message::<ResourceCollected>()
        .add_message::<TowerBuilt>()
        .add_message::<EnemySpawned>()
        .add_message::<EnemyKilled>()
        .add_message::<bevy::window::WindowCloseRequested>()
        .add_message::<AppExit>()
        .add_systems(
            Startup,
            (setup, spawn_village_health_bar, spawn_resource_counters),
        )
        .add_systems(Update, handle_menu_input.run_if(in_state(GameState::Menu)))
        .add_systems(
            Update,
            handle_game_input.run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, player_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, tower_building.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy_spawning.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, tower_shooting.run_if(in_state(GameState::Playing)))
        .add_systems(
            Update,
            tower_spawn_effect_system.run_if(in_state(GameState::Playing)),
        )
        // Observers for gameplay events (logging)
        .add_observer(on_resource_collected)
        .add_observer(on_tower_built)
        .add_observer(on_enemy_spawned)
        .add_observer(on_enemy_killed)
        // Camera system
        .add_systems(Update, camera_system.run_if(in_state(GameState::Playing)))
        // HUD systems
        .add_systems(
            Update,
            (
                village_health_hud,
                update_resource_counters,
                manage_collect_bar_ui,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                tower_spawn_effect_system,
                face_enemy_health_bars,
                update_enemy_health_bars,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Tree collection system
        .add_systems(Update, hold_to_collect.run_if(in_state(GameState::Playing)))
        // Window close handling - use force exit for immediate termination
        .add_systems(Update, force_exit_on_close)
        .run();
}
