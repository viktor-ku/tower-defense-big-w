use bevy::prelude::*;

/// Returns true if the `world` position is generally in front of the camera,
/// using a dot-product threshold against the camera's forward vector.
/// Typical `min_dot` values: 0.0 (half-space), 0.1-0.3 (narrower cone).
pub fn is_facing_world_pos(camera_transform: &GlobalTransform, world: Vec3, min_dot: f32) -> bool {
    let to_target = (world - camera_transform.translation()).normalize_or_zero();
    camera_transform.forward().dot(to_target) >= min_dot
}

/// Returns true if the `world` position projects into the current camera frustum
/// with an optional edge `margin` in NDC space. For example, margin 0.05 clips 5% near edges.
pub fn is_on_screen_ndc(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    world: Vec3,
    margin: f32,
) -> bool {
    if let Some(ndc) = camera.world_to_ndc(camera_transform, world) {
        let m = margin.max(0.0).min(0.49);
        ndc.z >= 0.0 && ndc.z <= 1.0 && ndc.x.abs() <= 1.0 - m && ndc.y.abs() <= 1.0 - m
    } else {
        false
    }
}

/// Convert a world position to logical (DPI-independent) viewport coordinates in pixels.
/// Returns None if the point cannot be projected.
pub fn world_to_viewport_logical(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
    world: Vec3,
) -> Option<Vec2> {
    let screen = camera.world_to_viewport(camera_transform, world).ok()?;
    Some(screen / window.resolution.scale_factor())
}

/// Transform a world-space point into camera-local space.
pub fn to_camera_space(world: Vec3, camera_transform: &GlobalTransform) -> Vec3 {
    camera_transform.affine().inverse().transform_point3(world)
}
