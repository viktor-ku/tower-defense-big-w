use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

mod components;
mod events;
mod setup;
mod systems;

use components::*;
use events::*;
use setup::*;
use systems::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Village Defender v0.1".into(),
                        resolution: (1920, 1080).into(),
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
                }),
            // Performance diagnostics
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        // Add explicit exit handling
        .add_systems(Update, bevy::window::close_when_requested)
        .add_systems(Update, bevy::window::exit_on_all_closed)
        .init_state::<GameState>()
        .insert_state(GameState::Playing)
        .add_message::<ResourceCollected>()
        .add_message::<WoodCollected>()
        .add_message::<TowerBuilt>()
        .add_message::<EnemySpawned>()
        .add_message::<EnemyKilled>()
        .add_message::<bevy::window::WindowCloseRequested>()
        .add_message::<AppExit>()
        .add_systems(Startup, setup)
        .add_systems(Update, handle_menu_input.run_if(in_state(GameState::Menu)))
        .add_systems(
            Update,
            handle_game_input.run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, player_movement.run_if(in_state(GameState::Playing)))
        .add_systems(
            Update,
            resource_collection.run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, tower_building.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy_spawning.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, tower_shooting.run_if(in_state(GameState::Playing)))
        .add_systems(Update, y_to_z_sort.run_if(in_state(GameState::Playing)))
        .add_systems(Update, day_night_cycle.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_events.run_if(in_state(GameState::Playing)))
        // Camera system
        .add_systems(Update, camera_system.run_if(in_state(GameState::Playing)))
        // Tree collection system
        .add_systems(
            Update,
            (tree_collection, handle_wood_collected_events).run_if(in_state(GameState::Playing)),
        )
        // Window close handling - use force exit for immediate termination
        .add_systems(Update, force_exit_on_close)
        .run();
}
