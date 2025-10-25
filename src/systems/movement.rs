use crate::components::*;
use crate::constants::Tunables;
use crate::systems::combat::projectiles::EnemyPreExplosion;
use bevy::input::keyboard::Key;
use bevy::prelude::*;

/// Moves the player using WASD/arrow keys at a fixed speed.
pub fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<Key>>,
    mut player_query: Query<&mut Transform, (With<Player>, With<IsoPlayer>)>,
    tunables: Res<Tunables>,
    mut log_accumulator: Local<f32>,
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

            // Debug: Log player position every few seconds without unsafe statics
            *log_accumulator += time.delta_secs();
            if *log_accumulator > 2.0 {
                info!("Player position: {:?}", transform.translation);
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
        Without<EnemyPreExplosion>,
    >,
    mut village_query: Query<&mut Village>,
    roads: Option<Res<RoadPaths>>,
    tunables: Res<Tunables>,
) {
    // Collision radius for village impact
    let village_collision_radius = tunables.village_collision_radius;

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
                    // Finished following the road, now move to village center
                    let to_center =
                        Vec3::new(0.0, transform.translation.y, 0.0) - transform.translation;
                    let dir = Vec3::new(to_center.x, 0.0, to_center.z).normalize_or_zero();
                    transform.translation += dir * enemy.speed * time.delta_secs();
                }
            }
        } else {
            // Fallback: Move towards center
            let to_center = Vec3::new(0.0, transform.translation.y, 0.0) - transform.translation;
            let dir = Vec3::new(to_center.x, 0.0, to_center.z).normalize_or_zero();
            transform.translation += dir * enemy.speed * time.delta_secs();
        }

        // Check if enemy actually hit the village block (much more precise collision)
        if Vec2::new(transform.translation.x, transform.translation.z).length()
            < village_collision_radius
        {
            if let Ok(mut village) = village_query.single_mut() {
                village.health = village.health.saturating_sub(10);
                info!(
                    "Village hit! Health remaining: {}/{}",
                    village.health, village.max_health
                );

                // Reset village health when destroyed (for easier testing)
                if village.health == 0 {
                    village.health = village.max_health;
                    info!(
                        "Village destroyed! Resetting health to {}",
                        village.max_health
                    );
                }
            }
            // Despawn enemy when it actually hits the village
            commands.entity(entity).despawn();
        }
    }
}
