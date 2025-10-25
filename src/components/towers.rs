use bevy::prelude::*;

/// Different kinds of towers selectable by the player.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TowerKind {
    Bow,
    Crossbow,
}

/// Combat tower with basic attack properties.
#[derive(Component)]
pub struct Tower {
    pub range: f32,
    pub damage: u32,
    /// Seconds between shots for this specific tower instance.
    pub fire_interval_secs: f32,
    /// Visual height of the tower, used for VFX spawn offsets.
    pub height: f32,
    /// Projectile speed for this tower's shots.
    pub projectile_speed: f32,
    pub last_shot: f32,
}

/// Marker for the in-progress tower preview (ghost).
#[derive(Component)]
pub struct TowerGhost;

/// Global selection state for tower building.
#[derive(Resource, Default)]
pub struct TowerBuildSelection {
    pub choice: Option<TowerKind>,
    /// Root UI entity for the drawer (if any), to allow clean despawn.
    pub drawer_root: Option<Entity>,
}
