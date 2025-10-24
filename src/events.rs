use bevy::prelude::*;

#[derive(Event, Message, Debug)]
pub struct ResourceCollected {
    pub kind: crate::components::HarvestableKind,
    pub amount: u32,
}

// WoodCollected removed; use ResourceCollected

#[derive(Event, Message, Debug)]
pub struct TowerBuilt {
    pub position: Vec3,
}

#[derive(Event, Message, Debug)]
pub struct EnemySpawned {
    pub position: Vec3,
}

#[derive(Event, Message, Debug)]
pub struct EnemyKilled {
    pub position: Vec3,
}
