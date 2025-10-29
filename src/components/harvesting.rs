use bevy::prelude::*;

/// Kinds of harvestable resources available in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarvestableKind {
    Wood,
    Rock,
}

/// Component for a harvestable resource node (e.g. tree, rock).
#[derive(Component, Debug, Clone, Copy)]
pub struct Harvestable {
    pub kind: HarvestableKind,
    pub amount: u32,
}

/// Marker for a tree resource node.
#[derive(Component)]
pub struct Tree;

/// Tracks the current hold-to-collect target and normalized progress [0,1].
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct CurrentCollectProgress {
    pub target: Option<Entity>,
    pub progress: f32,
}

/// Component for floating resource collection numbers.
#[derive(Component)]
pub struct ResourceNumber {
    pub timer: Timer,
    pub world_position: Vec3,
    pub ui_offset: Vec2,
}
