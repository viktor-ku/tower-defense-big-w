use bevy::prelude::*;

/// High-level app state controlling which systems run.
///
/// - Menu: main menu and non-gameplay screens
/// - Playing: active gameplay loop
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
}
