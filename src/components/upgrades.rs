use crate::components::towers::TowerKind;
use crate::components::upgrade_config::{TowerUpgradeConfig, UpgradeableStat};
use bevy::prelude::*;

/// Tracks purchased tower upgrades (the actual upgrade levels/values).
#[derive(Resource, Default)]
pub struct TowerUpgrades {
    pub bow_damage_level: u32,
    pub crossbow_damage_level: u32,
}

impl TowerUpgrades {
    /// Get damage bonus for a tower type at its current upgrade level.
    ///
    /// This is kept for backward compatibility. For new code, prefer using
    /// `calculate_stat_bonus` with `TowerUpgradeConfig`.
    pub fn get_damage_bonus(&self, kind: TowerKind) -> u32 {
        match kind {
            TowerKind::Bow => self.bow_damage_level * 5, // +5 damage per level
            TowerKind::Crossbow => self.crossbow_damage_level * 10, // +10 damage per level
        }
    }

    /// Get the current upgrade level for a tower type.
    pub fn get_level(&self, kind: TowerKind) -> u32 {
        match kind {
            TowerKind::Bow => self.bow_damage_level,
            TowerKind::Crossbow => self.crossbow_damage_level,
        }
    }

    /// Calculate bonuses using the upgrade configuration system.
    /// This method uses the declarative configuration instead of hardcoded values.
    ///
    /// # Example
    /// ```no_run
    /// use crate::components::{TowerKind, UpgradeableStat, TowerUpgrades, TowerUpgradeConfig};
    ///
    /// let upgrades = TowerUpgrades::default();
    /// let config = TowerUpgradeConfig::default();
    ///
    /// // Calculate damage bonus
    /// let damage_bonus = upgrades.calculate_stat_bonus(
    ///     TowerKind::Bow,
    ///     UpgradeableStat::Damage,
    ///     &config
    /// );
    ///
    /// // Calculate range bonus
    /// let range_bonus = upgrades.calculate_stat_bonus(
    ///     TowerKind::Bow,
    ///     UpgradeableStat::Range,
    ///     &config
    /// );
    /// ```
    pub fn calculate_stat_bonus(
        &self,
        kind: TowerKind,
        stat: UpgradeableStat,
        config: &TowerUpgradeConfig,
    ) -> f32 {
        let level = self.get_level(kind);
        config.calculate_bonus(kind, stat, level)
    }
}
