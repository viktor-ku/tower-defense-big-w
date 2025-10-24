use crate::components::*;
use crate::events::*;
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
pub fn hold_to_collect(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&Transform, &mut Player)>,
    harvestables: Query<(Entity, &Transform, &Harvestable)>,
    mut resource_events: MessageWriter<ResourceCollected>,
    mut commands: Commands,
    mut hold: Local<HoldCollectState>,
) {
    let Ok((player_transform, mut player)) = player_query.single_mut() else {
        hold.current_target = None;
        hold.elapsed_seconds = 0.0;
        return;
    };

    const COLLECT_RADIUS: f32 = 8.0;
    const HOLD_DURATION: f32 = 1.0;

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

    let is_holding = keyboard_input.pressed(KeyCode::KeyE);

    match (nearest, is_holding) {
        (Some((entity, _target_pos, harvestable)), true) => {
            if hold.current_target == Some(entity) {
                hold.elapsed_seconds += time.delta_secs();
            } else {
                hold.current_target = Some(entity);
                hold.elapsed_seconds = 0.0;
            }

            if hold.elapsed_seconds >= HOLD_DURATION {
                let collected = harvestable.amount;
                if collected > 0 {
                    match harvestable.kind {
                        HarvestableKind::Wood => {
                            player.wood += collected;
                        }
                        HarvestableKind::Rock => {
                            player.rock += collected;
                        }
                    }
                    resource_events.write(ResourceCollected {
                        kind: harvestable.kind,
                        amount: collected,
                    });
                }

                commands.entity(entity).despawn();
                hold.current_target = None;
                hold.elapsed_seconds = 0.0;
            }
        }
        _ => {
            hold.current_target = None;
            hold.elapsed_seconds = 0.0;
        }
    }
}
