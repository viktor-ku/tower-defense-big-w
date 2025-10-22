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

#[derive(Component)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub amount: u32,
}

#[derive(Component)]
pub struct Tree {
    pub wood_amount: u32,
    pub is_chopped: bool,
}

#[derive(Component)]
pub struct Tower {
    pub range: f32,
    pub damage: u32,
    pub last_shot: f32,
}

#[derive(Component)]
pub struct Enemy {
    pub health: u32,
    pub speed: f32,
}

#[derive(Component)]
pub struct Village {
    pub health: u32,
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
}

// 3D isometric markers
#[derive(Component)]
pub struct IsoPlayer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Rock,
}
