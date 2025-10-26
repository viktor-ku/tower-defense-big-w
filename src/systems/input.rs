use crate::components::*;
use bevy::input::keyboard::Key;
use bevy::prelude::*;

pub fn handle_menu_input(
    keyboard_input: Res<ButtonInput<Key>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(Key::Character("p".into())) {
        next_state.set(GameState::Playing);
    }
    if keyboard_input.just_pressed(Key::Escape) {
        std::process::exit(0);
    }
}

pub fn handle_game_input(
    keyboard_input: Res<ButtonInput<Key>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut building_mode_query: Query<&mut BuildingMode>,
    mut selection: ResMut<TowerBuildSelection>,
) {
    if keyboard_input.just_pressed(Key::Escape) {
        // If the tower drawer is open, Esc should cancel building instead of going to menu
        if selection.drawer_root.is_some() {
            for mut building_mode in building_mode_query.iter_mut() {
                building_mode.is_active = false;
            }
            selection.choice = None;
            return;
        } else {
            next_state.set(GameState::Menu);
        }
    }

    if keyboard_input.just_pressed(Key::Character("b".into())) {
        for mut building_mode in building_mode_query.iter_mut() {
            building_mode.is_active = !building_mode.is_active;
            if cfg!(debug_assertions) {
                info!("Building mode: {}", building_mode.is_active);
            }
        }
        // Clear selection so the drawer prompts again when entering build mode
        selection.choice = None;
    }
}
