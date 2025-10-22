use crate::components::*;
use bevy::prelude::*;

pub fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, (With<Player>, With<IsoPlayer>)>,
) {
    if let Ok(mut transform) = player_query.single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * 80.0 * time.delta_secs();

            // Debug: Log player position every few seconds
            static mut LAST_LOG: f32 = 0.0;
            unsafe {
                LAST_LOG += time.delta_secs();
                if LAST_LOG > 2.0 {
                    info!("Player position: {:?}", transform.translation);
                    LAST_LOG = 0.0;
                }
            }
        }
    }
}

pub fn enemy_movement(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &mut Transform, &Enemy)>,
    mut village_query: Query<&mut Village>,
) {
    // Village block is 8x8 units, so collision radius should be about 4-5 units
    const VILLAGE_COLLISION_RADIUS: f32 = 5.0;

    for (entity, mut transform, enemy) in enemy_query.iter_mut() {
        // Move towards village on XZ plane
        let to_center = Vec3::new(0.0, transform.translation.y, 0.0) - transform.translation;
        let dir = Vec3::new(to_center.x, 0.0, to_center.z).normalize_or_zero();
        transform.translation += dir * enemy.speed * time.delta_secs();

        // Check if enemy actually hit the village block (much more precise collision)
        if Vec2::new(transform.translation.x, transform.translation.z).length()
            < VILLAGE_COLLISION_RADIUS
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
