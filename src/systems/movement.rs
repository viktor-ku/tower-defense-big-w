use crate::audio::PlayerFootstepEvent;
use crate::components::*;
use crate::constants::Tunables;
use crate::systems::combat::projectiles::EnemyFadeOut;
use bevy::input::keyboard::Key;
use bevy::prelude::*;

/// Moves the player using WASD/arrow keys at a fixed speed.
pub fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<Key>>,
    mut player_query: Query<&mut Transform, (With<Player>, With<IsoPlayer>)>,
    tunables: Res<Tunables>,
    mut log_accumulator: Local<f32>,
    mut step_accumulator: Local<f32>,
    mut footstep_events: MessageWriter<PlayerFootstepEvent>,
) {
    if let Ok(mut transform) = player_query.single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(Key::Character("w".into()))
            || keyboard_input.pressed(Key::ArrowUp)
        {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(Key::Character("s".into()))
            || keyboard_input.pressed(Key::ArrowDown)
        {
            direction.z += 1.0;
        }
        if keyboard_input.pressed(Key::Character("a".into()))
            || keyboard_input.pressed(Key::ArrowLeft)
        {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(Key::Character("d".into()))
            || keyboard_input.pressed(Key::ArrowRight)
        {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * tunables.player_speed * time.delta_secs();

            // Footstep: emit at a regular cadence while moving
            *step_accumulator += time.delta_secs();
            let step_interval = 0.4_f32; // seconds per step (generic surface)
            if *step_accumulator >= step_interval {
                footstep_events.write(PlayerFootstepEvent {
                    position: transform.translation,
                });
                *step_accumulator = 0.0;
            }

            // Debug: Log player position every few seconds without unsafe statics
            *log_accumulator += time.delta_secs();
            if *log_accumulator > 2.0 {
                if cfg!(debug_assertions) {
                    info!("Player position: {:?}", transform.translation);
                }
                *log_accumulator = 0.0;
            }
        }
    }
}

/// Moves enemies along roads (if assigned) or toward the village center.
pub fn enemy_movement(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy_query: Query<
        (Entity, &mut Transform, &Enemy, Option<&mut PathFollower>),
        Without<EnemyFadeOut>,
    >,
    // Split queries to avoid Transform access conflicts; ensure disjoint via Without<Enemy>
    village_tf_query: Query<&Transform, (With<TownCenter>, Without<Enemy>)>,
    mut village_query: Query<&mut Village, With<TownCenter>>,
    roads: Option<Res<RoadPaths>>,
    tunables: Res<Tunables>,
) {
    // Collision radius for village impact
    let village_collision_radius = tunables.village_collision_radius;

    // Resolve current village/base position once (assumes single TownCenter)
    let village_pos = village_tf_query
        .single()
        .map(|tf| tf.translation)
        .unwrap_or(Vec3::ZERO);

    for (entity, mut transform, enemy, follower_opt) in enemy_query.iter_mut() {
        if let (Some(roads), Some(mut follower)) = (&roads, follower_opt) {
            if let Some(road) = roads.roads.get(follower.road_index) {
                if follower.next_index < road.len() {
                    let target = road[follower.next_index];
                    let to_target = Vec3::new(target.x, transform.translation.y, target.z)
                        - transform.translation;
                    let dir = Vec3::new(to_target.x, 0.0, to_target.z).normalize_or_zero();
                    transform.translation += dir * enemy.speed * time.delta_secs();
                    // Advance waypoint when close
                    if Vec2::new(to_target.x, to_target.z).length() < 1.0 {
                        follower.next_index += 1;
                    }
                } else {
                    // Finished following the road, now move to the actual village position
                    let to_village =
                        Vec3::new(village_pos.x, transform.translation.y, village_pos.z)
                            - transform.translation;
                    let dir = Vec3::new(to_village.x, 0.0, to_village.z).normalize_or_zero();
                    transform.translation += dir * enemy.speed * time.delta_secs();
                }
            }
        } else {
            // Fallback: Move towards the actual village position
            let to_village = Vec3::new(village_pos.x, transform.translation.y, village_pos.z)
                - transform.translation;
            let dir = Vec3::new(to_village.x, 0.0, to_village.z).normalize_or_zero();
            transform.translation += dir * enemy.speed * time.delta_secs();
        }

        // Check if enemy actually hit the village block (much more precise collision)
        let dx = transform.translation.x - village_pos.x;
        let dz = transform.translation.z - village_pos.z;
        if Vec2::new(dx, dz).length() < village_collision_radius {
            if let Ok(mut village) = village_query.single_mut() {
                village.health = village.health.saturating_sub(enemy.damage);
                if cfg!(debug_assertions) {
                    info!(
                        "Village hit! Health remaining: {}/{}",
                        village.health, village.max_health
                    );
                }

                // Reset village health when destroyed (for easier testing)
                if village.health == 0 {
                    village.health = village.max_health;
                    if cfg!(debug_assertions) {
                        info!(
                            "Village destroyed! Resetting health to {}",
                            village.max_health
                        );
                    }
                }
            }
            // Despawn enemy when it actually hits the village
            commands.entity(entity).despawn();
        }
    }
}
