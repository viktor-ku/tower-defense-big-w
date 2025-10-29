use bevy::prelude::*;

use crate::components::{GameState, TowerUpgradeConfig, TowerUpgrades};

pub mod definitions;
pub mod placement;
pub mod theme;
pub mod ui_menu;

/// Plugin that owns the build menu (Tab) and placement flow.
pub struct BuildPlugin;

impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ui_menu::BuildMenuState>()
            .init_resource::<definitions::BuildCatalog>()
            .init_resource::<ui_menu::CurrentCategory>()
            .init_resource::<TowerUpgrades>()
            .init_resource::<TowerUpgradeConfig>()
            .add_message::<ui_menu::ToggleBuildMenu>()
            .add_systems(
                OnEnter(GameState::Playing),
                definitions::ensure_default_catalog,
            )
            .add_systems(
                Update,
                (
                    ui_menu::toggle_build_menu_input,
                    ui_menu::manage_build_menu_ui,
                    ui_menu::handle_category_buttons,
                    ui_menu::handle_item_selection,
                    ui_menu::handle_upgrade_selection,
                ),
            );
    }
}
