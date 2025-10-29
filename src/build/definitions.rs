use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum BuildCategory {
    Towers,
    Upgrades,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BuildDefinitionId(pub &'static str);

#[derive(Clone, Debug)]
pub struct BuildDefinition {
    pub id: BuildDefinitionId,
    pub category: BuildCategory,
    pub display_name: &'static str,
    pub cost: u32,
    pub footprint_cells: UVec2,
}

#[derive(Clone, Debug)]
pub struct UpgradeDefinition {
    pub id: BuildDefinitionId,
    pub display_name: &'static str,
    pub gold_cost: u64,
    pub silver_cost: u64,
    pub tower_kind: crate::components::TowerKind,
}

#[derive(Resource, Default)]
pub struct BuildCatalog {
    pub items: Vec<BuildDefinition>,
    pub upgrades: Vec<UpgradeDefinition>,
}

impl BuildCatalog {
    pub fn ensure_defaults(&mut self) {
        if !self.items.is_empty() {
            return;
        }
        self.items = vec![
            BuildDefinition {
                id: BuildDefinitionId("bow_tower"),
                category: BuildCategory::Towers,
                display_name: "Bow Tower",
                cost: 10,
                footprint_cells: UVec2::new(1, 1),
            },
            BuildDefinition {
                id: BuildDefinitionId("crossbow_tower"),
                category: BuildCategory::Towers,
                display_name: "Crossbow Tower",
                cost: 20,
                footprint_cells: UVec2::new(1, 1),
            },
        ];
        self.upgrades = vec![
            UpgradeDefinition {
                id: BuildDefinitionId("bow_damage_upgrade"),
                display_name: "Bow Damage",
                gold_cost: 5,
                silver_cost: 10,
                tower_kind: crate::components::TowerKind::Bow,
            },
            UpgradeDefinition {
                id: BuildDefinitionId("crossbow_damage_upgrade"),
                display_name: "Crossbow Damage",
                gold_cost: 10,
                silver_cost: 20,
                tower_kind: crate::components::TowerKind::Crossbow,
            },
        ];
    }
}

pub fn ensure_default_catalog(mut catalog: ResMut<BuildCatalog>) {
    catalog.ensure_defaults();
}
