use crate::components::*;
use bevy::input::keyboard::Key;
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

pub fn handle_menu_input(
    keyboard_input: Res<ButtonInput<Key>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(Key::Character("p".into())) {
        next_state.set(GameState::Playing);
    }
    // Do not exit the game on Escape
}

pub fn handle_game_input(
    keyboard_input: Res<ButtonInput<Key>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut building_mode_query: Query<&mut BuildingMode>,
    mut selling_mode_query: Query<&mut SellingMode>,
    mut selection: ResMut<TowerBuildSelection>,
) {
    if keyboard_input.just_pressed(Key::Escape) || mouse_input.just_pressed(MouseButton::Right) {
        // Cancel building mode and any tower selection/preview
        let mut was_building = false;
        for mut building_mode in building_mode_query.iter_mut() {
            if building_mode.is_active {
                building_mode.is_active = false;
                was_building = true;
            }
        }
        // Also cancel selling mode
        for mut selling in selling_mode_query.iter_mut() {
            selling.is_active = false;
        }

        if was_building
            || selection.choice.is_some()
            || selection.hovered_choice.is_some()
            || selection.drawer_root.is_some()
        {
            selection.choice = None;
            selection.hovered_choice = None;
            // Drawer will be cleaned up by manage_tower_selection_drawer next frame
            return;
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

pub fn pause_toggle_input(
    keyboard_input: Res<ButtonInput<Key>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(Key::Space) {
        match state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}
