use bevy::prelude::*;

/// World-space road paths used for AI/path-following.
#[derive(Resource, Default, Debug, Clone)]
pub struct RoadPaths {
    /// Each road is a sequence of waypoints (centerline on XZ plane)
    pub roads: Vec<Vec<Vec3>>,
}

/// Component for entities that follow a given `RoadPaths` entry.
#[derive(Component, Debug, Clone, Copy)]
pub struct PathFollower {
    pub road_index: usize,
    pub next_index: usize,
}


