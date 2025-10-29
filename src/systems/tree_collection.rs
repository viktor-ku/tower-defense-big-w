use crate::components::*;
use crate::constants::Tunables;
use crate::events::*;
use bevy::input::keyboard::Key;
use bevy::prelude::*;

/// Local state for hold-to-collect interaction.
/// Tracks which target is being collected and how long E has been held.
#[derive(Default)]
pub struct HoldCollectState {
    current_target: Option<Entity>,
    elapsed_seconds: f32,
}

/// Hold-to-collect system for trees and rock resources.
///
/// Requirements:
/// - Player must be within 8 units of a target.
/// - Player must hold E for 1 second.
///
/// Behavior:
/// - Picks the nearest eligible target (tree with wood, rock with amount).
/// - On completion, grants resources, emits events, and despawns the target.
#[allow(clippy::too_many_arguments)]
pub fn hold_to_collect(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<Key>>,
    mut player_query: Query<(&Transform, &mut Player)>,
    harvestables: Query<(Entity, &Transform, &Harvestable)>,
    mut resource_events: MessageWriter<ResourceCollected>,
    mut current: ResMut<CurrentCollectProgress>,
    mut commands: Commands,
    mut hold: Local<HoldCollectState>,
) {
    let Ok((player_transform, mut player)) = player_query.single_mut() else {
        hold.current_target = None;
        hold.elapsed_seconds = 0.0;
        current.target = None;
        current.progress = 0.0;
        return;
    };

    const COLLECT_RADIUS: f32 = 8.0;
    const HOLD_DURATION: f32 = 1.0;

    // Only do the O(N) nearest scan when the key is held
    let is_holding = keyboard_input.pressed(Key::Character("e".into()));
    if !is_holding {
        hold.current_target = None;
        hold.elapsed_seconds = 0.0;
        current.target = None;
        current.progress = 0.0;
        return;
    }

    // Find nearest eligible target within radius among harvestables
    let mut nearest: Option<(Entity, Vec3, Harvestable)> = None;
    let mut best_dist_sq = f32::MAX;

    let player_pos = player_transform.translation;

    for (entity, transform, harvestable) in harvestables.iter() {
        if harvestable.amount == 0 {
            continue;
        }
        let d2 = player_pos.distance_squared(transform.translation);
        if d2 <= COLLECT_RADIUS * COLLECT_RADIUS && d2 < best_dist_sq {
            nearest = Some((entity, transform.translation, *harvestable));
            best_dist_sq = d2;
        }
    }

    match (nearest, is_holding) {
        (Some((entity, target_pos, harvestable)), true) => {
            if hold.current_target == Some(entity) {
                hold.elapsed_seconds += time.delta_secs();
            } else {
                hold.current_target = Some(entity);
                hold.elapsed_seconds = 0.0;
            }

            current.target = Some(entity);
            current.progress = (hold.elapsed_seconds / HOLD_DURATION).clamp(0.0, 1.0);

            if hold.elapsed_seconds >= HOLD_DURATION {
                let collected = harvestable.amount;
                if collected > 0 {
                    match harvestable.kind {
                        HarvestableKind::Wood => {
                            // Trees give 2 wood total
                            let actual_wood = 2;
                            player.wood += actual_wood;
                            resource_events.write(ResourceCollected {
                                kind: harvestable.kind,
                                amount: actual_wood,
                                position: target_pos,
                            });
                        }
                        HarvestableKind::Rock => {
                            // Rocks give 1 rock total
                            let actual_rock = 1;
                            player.rock += actual_rock;
                            resource_events.write(ResourceCollected {
                                kind: harvestable.kind,
                                amount: actual_rock,
                                position: target_pos,
                            });
                        }
                    }
                }

                commands.entity(entity).despawn();
                hold.current_target = None;
                hold.elapsed_seconds = 0.0;
                current.target = None;
                current.progress = 0.0;
            }
        }
        _ => {
            hold.current_target = None;
            hold.elapsed_seconds = 0.0;
            current.target = None;
            current.progress = 0.0;
        }
    }
}

/// System to spawn floating resource collection numbers when resources are collected.
pub fn resource_collected_spawn_text_system(
    mut commands: Commands,
    tunables: Res<Tunables>,
    mut events: MessageReader<ResourceCollected>,
    asset_server: Res<AssetServer>,
) {
    for evt in events.read() {
        // Use the position from the event
        let pos = evt.position + Vec3::new(0.0, tunables.damage_number_spawn_height, 0.0);

        // Choose a small random UI offset to prevent overlap
        let dir = rand::random::<u8>() % 4;
        let offset_px = match dir {
            0 => Vec2::new(10.0, 0.0),  // right
            1 => Vec2::new(-10.0, 0.0), // left
            2 => Vec2::new(0.0, 10.0),  // down
            _ => Vec2::new(0.0, -10.0), // up
        };

        // Choose color based on resource type
        let color = match evt.kind {
            HarvestableKind::Wood => Color::srgba(0.4, 0.8, 0.2, 0.9), // Green for wood
            HarvestableKind::Rock => Color::srgba(0.6, 0.6, 0.6, 0.9), // Gray for rock
        };

        commands.spawn((
            ResourceNumber {
                timer: Timer::from_seconds(tunables.damage_number_lifetime_secs, TimerMode::Once),
                world_position: pos,
                ui_offset: offset_px,
            },
            Text::new(format!("+{}", evt.amount)),
            TextFont {
                font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                font_size: tunables.damage_number_font_size,
                ..default()
            },
            TextColor(color),
        ));
    }
}

/// System to update and cleanup resource collection numbers.
pub fn resource_number_system(
    time: Res<Time>,
    mut commands: Commands,
    windows: Query<&Window>,
    cam_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut numbers: Query<(
        Entity,
        &mut ResourceNumber,
        &mut Node,
        &mut TextColor,
        &mut Visibility,
    )>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = cam_q.single() else {
        return;
    };

    let scale_factor = window.resolution.scale_factor();

    for (entity, mut number, mut node, mut color, mut visibility) in numbers.iter_mut() {
        number.timer.tick(time.delta());

        if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, number.world_position) {
            *visibility = Visibility::Visible;

            let margin = 10.0;

            // Convert to logical UI coordinates: top-left origin
            let logical_pos = screen_pos / scale_factor;
            node.left = Val::Px(logical_pos.x - margin + number.ui_offset.x);
            node.top = Val::Px(logical_pos.y - margin + number.ui_offset.y);
        } else {
            *visibility = Visibility::Hidden;
        }

        let duration = number.timer.duration().as_secs_f32().max(f32::EPSILON);
        let elapsed = number.timer.elapsed_secs();
        let progress = elapsed / duration;

        // Fade out over time
        let alpha = (1.0 - progress).clamp(0.0, 1.0);
        color.0.set_alpha(alpha);

        if number.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
