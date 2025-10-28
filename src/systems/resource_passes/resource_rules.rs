use bevy::prelude::*;

use super::PlacedByRule;

/// Global toggle for resource rule overlay execution.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ResourceRuleConfig {
    pub enabled: bool,
}

impl Default for ResourceRuleConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Convenience alias for the rock-specific rule marker if needed by callers.
pub type RulePlacedRock = PlacedByRule;
