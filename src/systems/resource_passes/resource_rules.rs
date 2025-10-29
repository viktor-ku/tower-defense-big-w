use bevy::prelude::*;


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

// Removed unused alias: callers can use PlacedByRule directly.
