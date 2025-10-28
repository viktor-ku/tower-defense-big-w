use bevy::prelude::*;

/// The central village object with health.
#[derive(Component)]
pub struct Village {
    pub health: u32,
    pub max_health: u32,
}

/// Global toggle for building placement mode.
#[derive(Component)]
pub struct BuildingMode {
    pub is_active: bool,
}

/// Global toggle for selling mode.
#[derive(Component)]
pub struct SellingMode {
    pub is_active: bool,
}

// Town markers for various world structures
#[derive(Component)]
pub struct TownCenter;

#[derive(Component)]
pub struct TownSquare;

#[derive(Component)]
pub struct Wall;
