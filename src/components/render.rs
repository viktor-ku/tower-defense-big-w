use bevy::prelude::*;

/// Marker for entities that should not be hidden by distance-based culling.
#[derive(Component)]
pub struct NoDistanceCull;
