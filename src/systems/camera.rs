use crate::components::*;
use bevy::prelude::*;

/// Settings for camera offset relative to the player.
#[derive(Resource, Clone)]
pub struct CameraSettings {
    pub offset: Vec3,
}

/// Positions the 3D camera at a fixed offset from the player and looks at the player.
pub fn camera_system(
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera3d>)>,
    settings: Res<CameraSettings>,
) {
    if let Ok(mut camera_transform) = camera_query.single_mut()
        && let Ok(player_transform) = player_query.single()
    {
        let player_pos = player_transform.translation;
        let offset = settings.offset;
        camera_transform.translation = player_pos + offset;
        camera_transform.look_at(player_pos, Vec3::Y);
    }
}
