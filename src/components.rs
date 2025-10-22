use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

#[derive(Component)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub amount: u32,
}

#[derive(Component)]
pub struct Tower {
    pub tower_type: TowerType,
    pub range: f32,
    pub damage: u32,
    pub last_shot: f32,
}

#[derive(Component)]
pub struct Enemy {
    pub health: u32,
    pub speed: f32,
    pub target: Vec3,
}

#[derive(Component)]
pub struct Village {
    pub health: u32,
    pub max_health: u32,
}

#[derive(Component)]
pub struct DayNight {
    pub is_day: bool,
    pub time_until_switch: f32,
    pub day_duration: f32,
    pub night_duration: f32,
}

#[derive(Component)]
pub struct BuildingMode {
    pub is_active: bool,
    pub tower_type: TowerType,
}

#[derive(Component)]
pub struct YSort;

#[derive(Component)]
pub struct Shadow;

// 3D isometric markers
#[derive(Component)]
pub struct IsoPlayer;

#[derive(Component)]
pub struct IsoEnemy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Wood,
    Rock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TowerType {
    Basic,
    Advanced,
}
