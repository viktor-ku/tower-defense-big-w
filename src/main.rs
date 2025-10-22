use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

mod components;
mod events;
mod systems;
mod setup;

use components::*;
use events::*;
use systems::*;
use setup::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Village Defender v0.1".into(),
                    resolution: (1024, 768).into(),
                    ..default()
                }),
                ..default()
            }),
            // Performance diagnostics
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .init_state::<GameState>()
        .insert_state(GameState::Playing)
        .add_message::<ResourceCollected>()
        .add_message::<TowerBuilt>()
        .add_message::<EnemySpawned>()
        .add_message::<EnemyKilled>()
        .add_systems(Startup, setup)
        .add_systems(Update, handle_menu_input.run_if(in_state(GameState::Menu)))
        .add_systems(Update, handle_game_input.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, resource_collection.run_if(in_state(GameState::Playing)))
        .add_systems(Update, tower_building.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy_spawning.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, tower_shooting.run_if(in_state(GameState::Playing)))
        .add_systems(Update, y_to_z_sort.run_if(in_state(GameState::Playing)))
        .add_systems(Update, day_night_cycle.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_events.run_if(in_state(GameState::Playing)))
        .run();
}