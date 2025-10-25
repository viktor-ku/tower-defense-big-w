use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Minion,
    Zombie,
    Boss,
}

impl EnemyKind {
    /// Returns (hp, damage_to_village, speed, cube_size)
    pub fn stats(self) -> (u32, u32, f32, f32) {
        match self {
            EnemyKind::Minion => (30, 5, 24.0, 0.8),
            EnemyKind::Zombie => (50, 10, 18.0, 1.2),
            EnemyKind::Boss => (100, 50, 12.0, 1.8),
        }
    }
}

/// Enemy unit with basic stats.
#[derive(Component)]
pub struct Enemy {
    pub health: u32,
    pub max_health: u32,
    pub speed: f32,
    pub damage: u32,
    pub kind: EnemyKind,
    /// Visual height (used for placing health bars above the unit)
    pub visual_height: f32,
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
