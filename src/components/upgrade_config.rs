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

/// Defines a level range and the bonus value for that range.
#[derive(Clone, Debug, Copy)]
pub struct UpgradeRange {
    /// Start of the range (inclusive)
    pub start_level: u32,
    /// End of the range (inclusive). Use `u32::MAX` for "and above".
    pub end_level: u32,
    /// Bonus value per level within this range
    pub bonus_per_level: f32,
}

impl UpgradeRange {
    /// Create a new upgrade range.
    pub fn new(start_level: u32, end_level: u32, bonus_per_level: f32) -> Self {
        Self {
            start_level,
            end_level,
            bonus_per_level,
        }
    }

    /// Create a range that extends to infinity (for "90+" type ranges).
    pub fn from_level(start_level: u32, bonus_per_level: f32) -> Self {
        Self {
            start_level,
            end_level: u32::MAX,
            bonus_per_level,
        }
    }

    /// Check if a level falls within this range.
    pub fn contains(&self, level: u32) -> bool {
        level >= self.start_level && level <= self.end_level
    }
}

/// Range-based upgrade configuration that allows defining bonuses per level range.
///
/// This allows you to say things like:
/// - Levels 0-19: +5 damage per level
/// - Levels 20-30: +15 damage per level
/// - Levels 31-89: +8 damage per level
/// - Levels 90+: +25 damage per level
///
/// The ranges are processed in order, and the first matching range is used.
#[derive(Clone, Debug)]
pub struct RangeBasedUpgrades {
    pub damage_ranges: Vec<UpgradeRange>,
    pub range_ranges: Vec<UpgradeRange>,
    pub fire_speed_ranges: Vec<UpgradeRange>,
    pub projectile_speed_ranges: Vec<UpgradeRange>,
}

impl Default for RangeBasedUpgrades {
    fn default() -> Self {
        Self {
            damage_ranges: vec![UpgradeRange::new(0, u32::MAX, 0.0)],
            range_ranges: vec![UpgradeRange::new(0, u32::MAX, 0.0)],
            fire_speed_ranges: vec![UpgradeRange::new(0, u32::MAX, 0.0)],
            projectile_speed_ranges: vec![UpgradeRange::new(0, u32::MAX, 0.0)],
        }
    }
}

impl RangeBasedUpgrades {
    /// Create a new range-based upgrade configuration with all stats at zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set damage bonuses using level ranges.
    ///
    /// # Example
    /// ```
    /// RangeBasedUpgrades::new()
    ///     .with_damage_ranges(vec![
    ///         UpgradeRange::new(0, 19, 5.0),      // Levels 0-19: +5 per level
    ///         UpgradeRange::new(20, 30, 15.0),   // Levels 20-30: +15 per level
    ///         UpgradeRange::new(31, 89, 8.0),   // Levels 31-89: +8 per level
    ///         UpgradeRange::from_level(90, 25.0), // Levels 90+: +25 per level
    ///     ])
    /// ```
    pub fn with_damage_ranges(mut self, ranges: Vec<UpgradeRange>) -> Self {
        self.damage_ranges = ranges;
        self
    }

    /// Set range bonuses using level ranges.
    pub fn with_range_ranges(mut self, ranges: Vec<UpgradeRange>) -> Self {
        self.range_ranges = ranges;
        self
    }

    /// Set fire speed bonuses using level ranges.
    pub fn with_fire_speed_ranges(mut self, ranges: Vec<UpgradeRange>) -> Self {
        self.fire_speed_ranges = ranges;
        self
    }

    /// Set projectile speed bonuses using level ranges.
    pub fn with_projectile_speed_ranges(mut self, ranges: Vec<UpgradeRange>) -> Self {
        self.projectile_speed_ranges = ranges;
        self
    }

    /// Calculate the total bonus for a stat at a given level.
    /// This sums up the bonuses from all ranges up to the current level.
    pub fn calculate_bonus(&self, stat: UpgradeableStat, level: u32) -> f32 {
        let ranges = match stat {
            UpgradeableStat::Damage => &self.damage_ranges,
            UpgradeableStat::Range => &self.range_ranges,
            UpgradeableStat::FireSpeed => &self.fire_speed_ranges,
            UpgradeableStat::ProjectileSpeed => &self.projectile_speed_ranges,
        };

        let mut total = 0.0;
        let mut current_level = 0u32;

        // Process each range in order
        for range in ranges {
            if current_level > level {
                break;
            }

            let range_start = range.start_level.max(current_level);
            let range_end = range.end_level.min(level);

            if range_start <= range_end {
                let levels_in_range = (range_end - range_start + 1) as f32;
                total += levels_in_range * range.bonus_per_level;
                current_level = range_end + 1;
            }
        }

        total
    }
}

/// Static upgrade table that defines exact bonuses per level (non-linear progression).
///
/// Each vector contains the total bonus at that level (level 0 = no upgrades, level 1 = first upgrade, etc.).
/// For example, `damage_by_level: vec![0, 5, 13, 25]` means:
/// - Level 0 (no upgrades): +0 damage
/// - Level 1: +5 damage
/// - Level 2: +13 damage total (not cumulative, this is the total)
/// - Level 3: +25 damage total
///
/// If a level exceeds the table, the last value is used.
#[derive(Clone, Debug)]
pub struct StaticUpgradeTable {
    pub damage_by_level: Vec<u32>,
    pub range_by_level: Vec<f32>,
    pub fire_speed_by_level: Vec<f32>, // Reduction in fire_interval_secs
    pub projectile_speed_by_level: Vec<f32>,
}

impl Default for StaticUpgradeTable {
    fn default() -> Self {
        Self {
            damage_by_level: vec![0],
            range_by_level: vec![0.0],
            fire_speed_by_level: vec![0.0],
            projectile_speed_by_level: vec![0.0],
        }
    }
}

impl StaticUpgradeTable {
    /// Create a new static upgrade table with all stats starting at zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the damage values for each level.
    /// The first element is level 0 (no upgrades), second is level 1, etc.
    ///
    /// # Example
    /// ```
    /// StaticUpgradeTable::new()
    ///     .with_damage_table(vec![0, 5, 13, 25, 40])
    /// ```
    pub fn with_damage_table(mut self, values: Vec<u32>) -> Self {
        self.damage_by_level = values;
        self
    }

    /// Set the range values for each level.
    pub fn with_range_table(mut self, values: Vec<f32>) -> Self {
        self.range_by_level = values;
        self
    }

    /// Set the fire speed values for each level (reductions in fire_interval_secs).
    pub fn with_fire_speed_table(mut self, values: Vec<f32>) -> Self {
        self.fire_speed_by_level = values;
        self
    }

    /// Set the projectile speed values for each level.
    pub fn with_projectile_speed_table(mut self, values: Vec<f32>) -> Self {
        self.projectile_speed_by_level = values;
        self
    }

    /// Get the bonus value for a specific stat at the given level.
    /// Returns 0.0 if the level is out of bounds (uses last value or 0).
    pub fn get_bonus(&self, stat: UpgradeableStat, level: u32) -> f32 {
        let level_index = level as usize;
        match stat {
            UpgradeableStat::Damage => *self
                .damage_by_level
                .get(level_index)
                .or_else(|| self.damage_by_level.last())
                .unwrap_or(&0) as f32,
            UpgradeableStat::Range => *self
                .range_by_level
                .get(level_index)
                .or_else(|| self.range_by_level.last())
                .unwrap_or(&0.0),
            UpgradeableStat::FireSpeed => *self
                .fire_speed_by_level
                .get(level_index)
                .or_else(|| self.fire_speed_by_level.last())
                .unwrap_or(&0.0),
            UpgradeableStat::ProjectileSpeed => *self
                .projectile_speed_by_level
                .get(level_index)
                .or_else(|| self.projectile_speed_by_level.last())
                .unwrap_or(&0.0),
        }
    }
}

/// Upgrade mode: linear scaling, static table, or range-based configuration.
#[derive(Clone, Debug)]
pub enum UpgradeMode {
    /// Linear scaling: bonuses are calculated as `level * bonus_per_level`.
    Linear(TowerUpgradeBonuses),
    /// Static table: bonuses are looked up directly from predefined values per level.
    Static(StaticUpgradeTable),
    /// Range-based: bonuses are defined per level range (e.g., "levels 20-30 use +15 per level").
    RangeBased(RangeBasedUpgrades),
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
    /// Create a new empty configuration.
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    /// Set the upgrade bonuses for a specific tower type using linear scaling.
    ///
    /// # Example
    /// ```no_run
    /// use crate::components::{TowerKind, TowerUpgradeBonuses, TowerUpgradeConfig};
    ///
    /// let mut config = TowerUpgradeConfig::new();
    /// config.set_upgrades_linear(
    ///     TowerKind::Bow,
    ///     TowerUpgradeBonuses::new()
    ///         .with_damage(5)
    ///         .with_range(2.0)
    ///         .with_fire_speed(0.1)
    ///         .with_projectile_speed(5.0)
    /// );
    /// ```
    pub fn set_upgrades_linear(&mut self, tower_kind: TowerKind, bonuses: TowerUpgradeBonuses) {
        self.configs
            .insert(tower_kind, UpgradeMode::Linear(bonuses));
    }

    /// Set the upgrade bonuses for a specific tower type using a static table.
    ///
    /// This allows you to define exact values per level without dynamic scaling.
    ///
    /// # Example
    /// ```no_run
    /// use crate::components::{TowerKind, StaticUpgradeTable, TowerUpgradeConfig};
    ///
    /// let mut config = TowerUpgradeConfig::new();
    /// config.set_upgrades_static(
    ///     TowerKind::Bow,
    ///     StaticUpgradeTable::new()
    ///         .with_damage_table(vec![0, 5, 13, 25, 40])  // Level 0: 0, Level 1: 5, Level 2: 13, etc.
    ///         .with_range_table(vec![0.0, 2.0, 5.0, 9.0])
    ///         .with_fire_speed_table(vec![0.0, 0.05, 0.12, 0.20])
    ///         .with_projectile_speed_table(vec![0.0, 3.0, 7.0, 12.0])
    /// );
    /// ```
    pub fn set_upgrades_static(&mut self, tower_kind: TowerKind, table: StaticUpgradeTable) {
        self.configs.insert(tower_kind, UpgradeMode::Static(table));
    }

    /// Set the upgrade bonuses for a specific tower type using range-based configuration.
    ///
    /// This allows you to define bonuses per level range, making it easy to specify
    /// different bonus rates for different level ranges.
    ///
    /// # Example
    /// ```no_run
    /// use crate::components::{TowerKind, RangeBasedUpgrades, UpgradeRange, TowerUpgradeConfig};
    ///
    /// let mut config = TowerUpgradeConfig::new();
    /// config.set_upgrades_ranges(
    ///     TowerKind::Bow,
    ///     RangeBasedUpgrades::new()
    ///         .with_damage_ranges(vec![
    ///             UpgradeRange::new(0, 19, 5.0),      // Levels 0-19: +5 per level
    ///             UpgradeRange::new(20, 30, 15.0),    // Levels 20-30: +15 per level
    ///             UpgradeRange::new(31, 89, 8.0),     // Levels 31-89: +8 per level
    ///             UpgradeRange::from_level(90, 25.0), // Levels 90+: +25 per level
    ///         ])
    ///         .with_range_ranges(vec![
    ///             UpgradeRange::new(0, 49, 2.0),
    ///             UpgradeRange::from_level(50, 5.0),
    ///         ])
    /// );
    /// ```
    pub fn set_upgrades_ranges(&mut self, tower_kind: TowerKind, ranges: RangeBasedUpgrades) {
        self.configs
            .insert(tower_kind, UpgradeMode::RangeBased(ranges));
    }

    /// Calculate the bonus for a specific stat at the specified upgrade level.
    /// Works with linear scaling, static tables, and range-based configurations.
    pub fn calculate_bonus(&self, tower_kind: TowerKind, stat: UpgradeableStat, level: u32) -> f32 {
        self.configs
            .get(&tower_kind)
            .map(|mode| match mode {
                UpgradeMode::Linear(bonuses) => bonuses.calculate_bonus(stat, level),
                UpgradeMode::Static(table) => table.get_bonus(stat, level),
                UpgradeMode::RangeBased(ranges) => ranges.calculate_bonus(stat, level),
            })
            .unwrap_or(0.0)
    }
}

/// Helper function to easily set up upgrade configurations declaratively.
///
/// This function is intended to be called during app initialization to configure
/// upgrade bonuses. It provides a clean, declarative API for defining how upgrades work.
///
/// # Example
/// ```no_run
/// use crate::components::{TowerKind, TowerUpgradeBonuses, TowerUpgradeConfig};
///
/// fn setup_upgrades(mut config: ResMut<TowerUpgradeConfig>) {
///     // Configure Bow tower upgrades
///     config.set_upgrades(
///         TowerKind::Bow,
///         TowerUpgradeBonuses::new()
///             .with_damage(5)           // +5 damage per level
///             .with_range(2.0)          // +2.0 range per level
///             .with_fire_speed(0.05)    // -0.05 seconds per level (faster firing)
///             .with_projectile_speed(3.0) // +3.0 projectile speed per level
///     );
///
///     // Configure Crossbow tower upgrades
///     config.set_upgrades(
///         TowerKind::Crossbow,
///         TowerUpgradeBonuses::new()
///             .with_damage(10)           // +10 damage per level
///             .with_range(3.0)          // +3.0 range per level
///             .with_fire_speed(0.1)     // -0.1 seconds per level
///             .with_projectile_speed(5.0) // +5.0 projectile speed per level
///     );
/// }
/// ```
#[allow(dead_code)]
pub fn setup_upgrade_config(_config: ResMut<TowerUpgradeConfig>) {
    // This function can be customized to set up upgrades declaratively
    // The default implementation in TowerUpgradeConfig::default() is already
    // set up, but you can override it here or add new tower types.
}
