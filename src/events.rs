use bevy::prelude::*;

#[derive(Event, Message, Debug)]
pub struct ResourceCollected {
    pub resource_type: crate::components::ResourceType,
    pub amount: u32,
}

#[derive(Event, Message, Debug)]
pub struct WoodCollected {
    pub amount: u32,
    pub tree_position: Vec3,
}

#[derive(Event, Message, Debug)]
pub struct TowerBuilt {
    pub position: Vec3,
    pub tower_type: crate::components::TowerType,
}

#[derive(Event, Message, Debug)]
pub struct EnemySpawned {
    pub position: Vec3,
}

#[derive(Event, Message, Debug)]
pub struct EnemyKilled {
    pub position: Vec3,
}
