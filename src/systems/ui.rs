use std::ops::Mul;

use crate::components::*;
use crate::events::*;
use bevy::prelude::*;

pub fn handle_events(
    mut resource_events: MessageReader<ResourceCollected>,
    mut tower_events: MessageReader<TowerBuilt>,
    mut enemy_spawned_events: MessageReader<EnemySpawned>,
    mut enemy_killed_events: MessageReader<EnemyKilled>,
) {
    for event in resource_events.read() {
        info!(
            "Resource collected: {:?} x{}",
            event.resource_type, event.amount
        );
    }

    for event in tower_events.read() {
        info!("Tower built at: {:?}", event.position);
    }

    for event in enemy_spawned_events.read() {
        info!("Enemy spawned at: {:?}", event.position);
    }

    for event in enemy_killed_events.read() {
        info!("Enemy killed at: {:?}", event.position);
    }
}

// UI Health bar system for village - updates persistent HUD fill width
pub fn village_health_hud(
    windows: Query<&Window>,
    village_query: Query<&Village>,
    mut fill_query: Query<&mut Node, With<HealthBar>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    if let Ok(village) = village_query.single() {
        let health_percentage = village.health as f32 / village.max_health as f32;
        let total_width_px = window.width() * 0.6; // 60% of screen
        let fill_width_px = total_width_px * health_percentage.clamp(0.0, 1.0);

        for mut node in fill_query.iter_mut() {
            // The green bar should always start from the left and shrink from the right
            // So we only change the width, keeping left position fixed
            node.width = Val::Px(fill_width_px);
        }
    }
}

// Component to mark health bar entities
#[derive(Component)]
pub struct HealthBar;

// Spawn persistent on-screen HUD: background + fill (marked with HealthBar)
pub fn spawn_village_health_bar(mut commands: Commands) {
    // Root container: centered horizontally using left: 20% and width: 60%
    commands
        .spawn((
            Node {
                left: Val::Percent(20.0),
                top: Val::Px(20.0),
                width: Val::Percent(60.0),
                height: Val::Px(40.0),
                ..default()
            },
            // White outline
            BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.85, 0.2)),
                HealthBar,
            ));
        });
}
