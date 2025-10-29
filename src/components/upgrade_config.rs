use crate::components::towers::TowerKind;
use bevy::prelude::*;
use std::collections::HashMap;

/// Stat types that can be upgraded.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum UpgradeableStat {
    Damage,
    Range,
    FireSpeed, // Reduces fire_interval_secs (higher = faster)
    ProjectileSpeed,
}

/// Configuration for how upgrades affect a tower's stats.
///
/// Each field represents the bonus per upgrade level.
/// For example, `damage_per_level: 5` means each upgrade level adds 5 damage.
#[derive(Clone, Debug)]
pub struct TowerUpgradeBonuses {
    pub damage_per_level: u32,
    pub range_per_level: f32,
    pub fire_speed_per_level: f32, // Reduction in fire_interval_secs per level
    pub projectile_speed_per_level: f32,
}

impl Default for TowerUpgradeBonuses {
    fn default() -> Self {
        Self {
            damage_per_level: 0,
            range_per_level: 0.0,
            fire_speed_per_level: 0.0,
            projectile_speed_per_level: 0.0,
        }
    }
}

impl TowerUpgradeBonuses {
    /// Create a new upgrade bonuses configuration with all values set to zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set damage bonus per level.
    pub fn with_damage(mut self, bonus_per_level: u32) -> Self {
        self.damage_per_level = bonus_per_level;
        self
    }

    /// Set range bonus per level.
    pub fn with_range(mut self, bonus_per_level: f32) -> Self {
        self.range_per_level = bonus_per_level;
        self
    }

    /// Set fire speed bonus per level (reduces fire_interval_secs).
    /// Higher values mean faster firing (lower interval).
    /// Example: `with_fire_speed(0.1)` reduces fire_interval_secs by 0.1 per level.
    pub fn with_fire_speed(mut self, reduction_per_level: f32) -> Self {
        self.fire_speed_per_level = reduction_per_level;
        self
    }

    /// Set projectile speed bonus per level.
    pub fn with_projectile_speed(mut self, bonus_per_level: f32) -> Self {
        self.projectile_speed_per_level = bonus_per_level;
        self
    }

    /// Calculate the total bonus for a given stat at the specified upgrade level.
    pub fn calculate_bonus(&self, stat: UpgradeableStat, level: u32) -> f32 {
        let multiplier = level as f32;
        match stat {
            UpgradeableStat::Damage => self.damage_per_level as f32 * multiplier,
            UpgradeableStat::Range => self.range_per_level * multiplier,
            UpgradeableStat::FireSpeed => self.fire_speed_per_level * multiplier,
            UpgradeableStat::ProjectileSpeed => self.projectile_speed_per_level * multiplier,
        }
    }
}

/// Upgrade mode: linear scaling, static table, or range-based configuration.
#[derive(Clone, Debug)]
pub enum UpgradeMode {
    /// Linear scaling: bonuses are calculated as `level * bonus_per_level`.
    Linear(TowerUpgradeBonuses),
}

/// Resource that stores upgrade bonus configurations for each tower type.
#[derive(Resource)]
pub struct TowerUpgradeConfig {
    configs: HashMap<TowerKind, UpgradeMode>,
}

impl Default for TowerUpgradeConfig {
    fn default() -> Self {
        let mut configs = HashMap::new();

        // Default configurations for each tower type using linear scaling
        // These can be easily modified or overridden
        configs.insert(
            TowerKind::Bow,
            UpgradeMode::Linear(
                TowerUpgradeBonuses::new()
                    .with_damage(5)
                    .with_range(0.0)
                    .with_fire_speed(0.0)
                    .with_projectile_speed(0.0),
            ),
        );

        configs.insert(
            TowerKind::Crossbow,
            UpgradeMode::Linear(
                TowerUpgradeBonuses::new()
                    .with_damage(10)
                    .with_range(0.0)
                    .with_fire_speed(0.0)
                    .with_projectile_speed(0.0),
            ),
        );

        Self { configs }
    }
}

impl TowerUpgradeConfig {
    // Removed unused constructors and setters; use Default and direct config map if needed

    /// Calculate the bonus for a specific stat at the specified upgrade level.
    /// Works with linear scaling.
    pub fn calculate_bonus(&self, tower_kind: TowerKind, stat: UpgradeableStat, level: u32) -> f32 {
        self.configs
            .get(&tower_kind)
            .map(|mode| match mode {
                UpgradeMode::Linear(bonuses) => bonuses.calculate_bonus(stat, level),
            })
            .unwrap_or(0.0)
    }
}

// Removed unused setup_upgrade_config helper and related docs
