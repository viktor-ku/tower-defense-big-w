use crate::components::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct CameraFollow {
    pub offset: Vec3,
    pub follow_speed: f32,
    pub edge_threshold: f32,
}

pub fn camera_follow_player(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &CameraFollow), With<Camera3d>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera3d>)>,
) {
    if let Ok((mut camera_transform, camera_follow)) = camera_query.single_mut() {
        if let Ok(player_transform) = player_query.single() {
            // Calculate target camera position
            let target_position = player_transform.translation + camera_follow.offset;

            // Smooth camera movement
            let current_pos = camera_transform.translation;
            let new_position = current_pos.lerp(
                target_position,
                camera_follow.follow_speed * time.delta_secs(),
            );

            camera_transform.translation = new_position;
        }
    }
}

pub fn camera_edge_movement(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    player_query: Query<&Transform, With<Player>>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.single() {
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            if let Ok(player_transform) = player_query.single() {
                let window_size = window.resolution.clone();
                let mouse_pos = window.cursor_position().unwrap_or(Vec2::new(
                    window_size.width() / 2.0,
                    window_size.height() / 2.0,
                ));

                // Define edge zones (percentage of screen from edges)
                let edge_zone = 0.15; // 15% from each edge
                let edge_threshold_x = window_size.width() * edge_zone;
                let edge_threshold_y = window_size.height() * edge_zone;

                let mut camera_movement = Vec3::ZERO;
                let move_speed = 200.0 * time.delta_secs();

                // Check left edge
                if mouse_pos.x < edge_threshold_x {
                    camera_movement.x -= move_speed;
                }
                // Check right edge
                if mouse_pos.x > window_size.width() - edge_threshold_x {
                    camera_movement.x += move_speed;
                }
                // Check top edge
                if mouse_pos.y < edge_threshold_y {
                    camera_movement.z += move_speed; // Z is forward in our coordinate system
                }
                // Check bottom edge
                if mouse_pos.y > window_size.height() - edge_threshold_y {
                    camera_movement.z -= move_speed;
                }

                // Apply camera movement
                camera_transform.translation += camera_movement;
            }
        }
    }
}

pub fn setup_camera_follow(mut commands: Commands) {
    // Add camera follow component to existing camera
    // This will be called after the camera is spawned in setup
}
