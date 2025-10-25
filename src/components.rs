use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
}

#[derive(Component)]
pub struct Player {
    pub wood: u32,
    pub rock: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarvestableKind {
    Wood,
    Rock,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Harvestable {
    pub kind: HarvestableKind,
    pub amount: u32,
}

#[derive(Component)]
pub struct Tree;

#[derive(Component)]
pub struct Tower {
    pub range: f32,
    pub damage: u32,
    pub last_shot: f32,
}

/// Marker for the in-progress tower preview (ghost).
#[derive(Component)]
pub struct TowerGhost;

#[derive(Component)]
pub struct Enemy {
    pub health: u32,
    pub max_health: u32,
    pub speed: f32,
}

#[derive(Component)]
pub struct EnemyHealthBarRoot;

#[derive(Component)]
pub struct EnemyHealthBarFill {
    pub max_width: f32,
    pub owner: Entity,
    pub last_ratio: f32,
}

#[derive(Component)]
pub struct Village {
    pub health: u32,
    pub max_health: u32,
}

// DayNight removed

#[derive(Component)]
pub struct BuildingMode {
    pub is_active: bool,
}

// 3D isometric markers
#[derive(Component)]
pub struct IsoPlayer;

// ResourceType removed in favor of HarvestableKind

// Road pathing
#[derive(Resource, Default, Debug, Clone)]
pub struct RoadPaths {
    pub roads: Vec<Vec<Vec3>>, // Each road is a sequence of waypoints (centerline on XZ)
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PathFollower {
    pub road_index: usize,
    pub next_index: usize,
}

/// Tracks the current hold-to-collect target and normalized progress [0,1].
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct CurrentCollectProgress {
    pub target: Option<Entity>,
    pub progress: f32,
}
