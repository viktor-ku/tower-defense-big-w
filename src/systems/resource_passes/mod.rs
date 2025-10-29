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

pub use crate::core::geometry::{distance_to_polyline_xz, sample_point_on_polyline_xz};
