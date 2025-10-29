use bevy::prelude::*;

use crate::components::roads::RoadPaths;

pub mod resource_rules;
pub use resource_rules::*;
pub mod rocks_along_road;
pub use rocks_along_road::*;
pub mod town_square_exclusion;
pub use town_square_exclusion::*;

/// System set for resource post-processing passes (run after random chunk spawns).
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ResourcePassSet {
    /// Apply rule-driven resource overlays (e.g., ensure rocks along road)
    Apply,
}

/// Marker for entities placed by a specific rule. `id` is a stable per-rule identifier.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlacedByRule {
    pub id: u64,
}

/// Root plugin that defines the scheduling set for resource passes.
pub struct ResourcePassesPlugin;

impl Plugin for ResourcePassesPlugin {
    fn build(&self, app: &mut App) {
        // Ensure the set exists; individual rule plugins will register into it.
        app.configure_sets(PostUpdate, ResourcePassSet::Apply);

        // Keep RoadPaths available for passes; no systems here.
        if app.world().get_resource::<RoadPaths>().is_none() {
            // Do nothing; RoadPaths is inserted in setup when roads are generated.
        }
    }
}

// ----------------------- Shared helpers -----------------------

/// Compute the minimum distance (in XZ) from a point to a polyline.
pub fn distance_to_polyline_xz(point: Vec3, path: &[Vec3]) -> f32 {
    if path.len() < 2 {
        return f32::INFINITY;
    }
    let p = Vec2::new(point.x, point.z);
    let mut best = f32::INFINITY;
    for seg in path.windows(2) {
        let a = Vec2::new(seg[0].x, seg[0].z);
        let b = Vec2::new(seg[1].x, seg[1].z);
        let ab = b - a;
        let ab_len2 = ab.length_squared();
        if ab_len2 <= f32::EPSILON {
            best = best.min(p.distance(a));
            continue;
        }
        let t = ((p - a).dot(ab) / ab_len2).clamp(0.0, 1.0);
        let closest = a + ab * t;
        best = best.min(p.distance(closest));
    }
    best
}

/// Sample a point along the polyline centerline, approximately uniform over segments.
pub fn sample_point_on_polyline_xz(path: &[Vec3], t: f32) -> (Vec3, Vec3) {
    // Returns (point, forward_dir). If path invalid, returns zeros.
    if path.len() < 2 {
        return (Vec3::ZERO, Vec3::X);
    }
    let mut segment_index = ((path.len() - 1) as f32 * t).floor() as usize;
    if segment_index >= path.len() - 1 {
        segment_index = path.len() - 2;
    }
    let local_t = (t * (path.len() - 1) as f32) - segment_index as f32;
    let a = path[segment_index];
    let b = path[segment_index + 1];
    let pos = a.lerp(b, local_t);
    let dir = (b - a).normalize_or_zero();
    (pos, dir)
}
