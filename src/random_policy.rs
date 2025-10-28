use bevy::prelude::*;

/// Centralized toggles for which systems should be deterministic (seeded)
/// versus non-deterministic (fresh random each run/event).
#[derive(Resource, Debug, Clone, Copy)]
pub struct RandomizationPolicy {
    /// Whether wave composition (mix and order of enemies) is seeded.
    pub wave_composition_seeded: bool,
    /// Whether enemy spawn entrance selection / ring angle is seeded.
    pub enemy_spawn_selection_seeded: bool,
    /// Whether town layout decisions (gate side, offsets, etc.) are seeded.
    pub town_layout_seeded: bool,
    /// Whether road generation/pathing from gate to base is seeded.
    pub road_generation_seeded: bool,
    /// Whether chunk content (trees/rocks distribution) is seeded.
    pub chunk_content_seeded: bool,
    /// Whether rule-based resource passes are seeded.
    pub resource_rules_seeded: bool,
}

impl Default for RandomizationPolicy {
    fn default() -> Self {
        RandomizationPolicy {
            wave_composition_seeded: true,
            enemy_spawn_selection_seeded: true,
            town_layout_seeded: true,
            road_generation_seeded: true,
            chunk_content_seeded: true,
            resource_rules_seeded: true,
        }
    }
}
