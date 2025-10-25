use bevy::prelude::*;

/// Inventory and player-related data.
#[derive(Component)]
pub struct Player {
    pub wood: u32,
    pub rock: u32,
}

/// Marker for the 3D player entity used in the world.
#[derive(Component)]
pub struct IsoPlayer;


