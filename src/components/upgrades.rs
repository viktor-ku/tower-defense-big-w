use crate::components::towers::TowerKind;
// Removed unused imports of upgrade configuration helpers
use bevy::prelude::*;

/// Tracks purchased tower upgrades (the actual upgrade levels/values).
#[derive(Resource, Default)]
pub struct TowerUpgrades {
    pub bow_damage_level: u32,
    pub crossbow_damage_level: u32,
}

impl TowerUpgrades {
    /// Get the current upgrade level for a tower type.
    pub fn get_level(&self, kind: TowerKind) -> u32 {
        match kind {
            TowerKind::Bow => self.bow_damage_level,
            TowerKind::Crossbow => self.crossbow_damage_level,
        }
    }
}
