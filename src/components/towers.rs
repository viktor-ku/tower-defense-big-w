use bevy::prelude::*;

/// Different kinds of towers selectable by the player.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TowerKind {
    Bow,
    Crossbow,
}

impl TowerKind {
    pub fn cost(self) -> (u32, u32) {
        match self {
            TowerKind::Bow => (3, 1),
            TowerKind::Crossbow => (10, 3),
        }
    }
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

/// Marker storing which kind this built tower is, used for selling/refunds.
#[derive(Component, Copy, Clone, Debug)]
pub struct BuiltTower {
    pub kind: TowerKind,
}

/// Marker for the in-progress tower preview (ghost).
#[derive(Component)]
pub struct TowerGhost;

/// Global selection state for tower building.
#[derive(Resource, Default)]
pub struct TowerBuildSelection {
    pub choice: Option<TowerKind>,
}

/// Component for persistent damage label displayed below towers.
#[derive(Component)]
pub struct TowerDamageLabel {
    pub tower_entity: Entity,
    pub world_offset: Vec3,
}

/// Marker on the tower entity indicating a damage label has been spawned.
#[derive(Component)]
pub struct HasTowerDamageLabel;
