use bevy::prelude::*;

/// Enemy unit with basic stats.
#[derive(Component)]
pub struct Enemy {
    pub health: u32,
    pub max_health: u32,
    pub speed: f32,
}

/// Marker for the health bar root entity attached to an enemy.
#[derive(Component)]
pub struct EnemyHealthBarRoot;

/// Fill bar component tracking width constraints and owner entity.
#[derive(Component)]
pub struct EnemyHealthBarFill {
    pub max_width: f32,
    pub owner: Entity,
    pub last_ratio: f32,
}


