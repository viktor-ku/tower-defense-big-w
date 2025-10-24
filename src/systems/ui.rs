use crate::components::*;
use crate::events::*;
use bevy::prelude::*;

/// Logs gameplay events to the console.
pub fn handle_events(
    mut resource_events: MessageReader<ResourceCollected>,
    mut tower_events: MessageReader<TowerBuilt>,
    mut enemy_spawned_events: MessageReader<EnemySpawned>,
    mut enemy_killed_events: MessageReader<EnemyKilled>,
) {
    for event in resource_events.read() {
        info!("Resource collected: {:?} x{}", event.kind, event.amount);
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

/// Updates the persistent HUD health bar for the village.
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

/// Marker component for the health bar fill node.
#[derive(Component)]
pub struct HealthBar;

/// Spawns the persistent on-screen HUD with a background and a fill node.
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

/// Marker for the wood counter text node.
#[derive(Component)]
pub struct WoodCounterText;

/// Marker for the rock counter text node.
#[derive(Component)]
pub struct RockCounterText;

/// Spawns resource counters (wood and rock) at the top-left of the screen.
pub fn spawn_resource_counters(mut commands: Commands) {
    commands
        .spawn((
            Node {
                left: Val::Px(20.0),
                top: Val::Px(70.0),
                width: Val::Auto,
                height: Val::Auto,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Wood: 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                WoodCounterText,
            ));

            parent.spawn((
                Node {
                    height: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));

            parent.spawn((
                Text::new("Rock: 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                RockCounterText,
            ));
        });
}

/// Updates the on-screen resource counters from the Player inventory.
pub fn update_resource_counters(
    player_q: Query<&Player>,
    mut wood_q: Query<&mut Text, (With<WoodCounterText>, Without<RockCounterText>)>,
    mut rock_q: Query<&mut Text, (With<RockCounterText>, Without<WoodCounterText>)>,
) {
    if let Ok(player) = player_q.single() {
        if let Ok(mut wood_text) = wood_q.single_mut() {
            *wood_text = Text::new(format!("Wood: {}", player.wood));
        }
        if let Ok(mut rock_text) = rock_q.single_mut() {
            *rock_text = Text::new(format!("Rock: {}", player.rock));
        }
    }
}
