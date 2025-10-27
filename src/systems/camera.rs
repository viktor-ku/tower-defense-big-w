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
    mut last: Local<Option<(Vec3, Vec3)>>, // (last_player_pos, last_offset)
) {
    if let Ok(mut camera_transform) = camera_query.single_mut()
        && let Ok(player_transform) = player_query.single()
    {
        let player_pos = player_transform.translation;
        let offset = settings.offset;

        let should_update = match *last {
            Some((lp, lo)) => {
                (player_pos - lp).length_squared() > 0.0001
                    || (offset - lo).length_squared() > 0.0001
            }
            None => true,
        };

        if should_update {
            camera_transform.translation = player_pos + offset;
            camera_transform.look_at(player_pos, Vec3::Y);
            *last = Some((player_pos, offset));
        }
    }
}
