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
    mut enemy_query: Query<(&mut Transform, &Enemy)>,
    mut village_query: Query<&mut Village>,
) {
    const VILLAGE_SIZE: f32 = 50.0;

    for (mut transform, enemy) in enemy_query.iter_mut() {
        // Move towards village on XZ plane
        let to_center = Vec3::new(0.0, transform.translation.y, 0.0) - transform.translation;
        let dir = Vec3::new(to_center.x, 0.0, to_center.z).normalize_or_zero();
        transform.translation += dir * enemy.speed * time.delta_secs();

        // Check if enemy reached village
        if Vec2::new(transform.translation.x, transform.translation.z).length() < VILLAGE_SIZE {
            if let Ok(mut village) = village_query.single_mut() {
                village.health = village.health.saturating_sub(10);
            }
            // Despawn enemy (this would be handled by a separate system in a real game)
        }
    }
}
