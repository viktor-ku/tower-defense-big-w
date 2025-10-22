use crate::components::*;
use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct CameraSettings {
    pub offset: Vec3,
    pub yaw_degrees: f32,
    pub pitch_degrees: f32,
    pub lock_rotation: bool,
}

pub fn camera_system(
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera3d>)>,
    settings: Res<CameraSettings>,
) {
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        if let Ok(player_transform) = player_query.single() {
            // Strict fixed offset
            camera_transform.translation = player_transform.translation + settings.offset;

            // Always look at the player
            camera_transform.look_at(player_transform.translation, Vec3::Y);
        }
    }
}
