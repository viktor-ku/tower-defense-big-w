use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum BuildCategory {
    Towers,
    Buildings,
    Plans,
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

#[derive(Resource, Default)]
pub struct BuildCatalog {
    pub items: Vec<BuildDefinition>,
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
    }
}

pub fn ensure_default_catalog(mut catalog: ResMut<BuildCatalog>) {
    catalog.ensure_defaults();
}
