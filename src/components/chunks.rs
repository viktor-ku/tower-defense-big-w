use bevy::prelude::*;

/// Marker for a chunk root entity (used by chunk loading/unloading systems).
#[derive(Component, Debug, Clone, Copy)]
pub struct ChunkRoot;
