use bevy::prelude::*;

/// Combat tower with basic attack properties.
#[derive(Component)]
pub struct Tower {
    pub range: f32,
    pub damage: u32,
    pub last_shot: f32,
}

/// Marker for the in-progress tower preview (ghost).
#[derive(Component)]
pub struct TowerGhost;


