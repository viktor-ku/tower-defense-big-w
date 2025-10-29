use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy_state::app::StatesPlugin;

use td::components::harvesting::{Harvestable, HarvestableKind};
use td::components::roads::RoadPaths;
use td::components::state::GameState;
use td::constants::Tunables;
use td::random_policy::RandomizationPolicy;
use td::systems::chunks::ChunkAssets;
use td::systems::resource_passes::rocks_along_road::RocksAlongRoadPassPlugin;
use td::systems::resource_passes::{ResourcePassesPlugin, RocksAlongRoadConfig};

fn simple_straight_road() -> RoadPaths {
    RoadPaths {
        roads: vec![vec![
            Vec3::new(-20.0, 0.0, -20.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(20.0, 0.0, 20.0),
        ]],
    }
}

fn count_rule_rocks(world: &mut World) -> usize {
    world
        .query::<(
            &Harvestable,
            Option<&td::systems::resource_passes::PlacedByRule>,
        )>()
        .iter(world)
        .filter(|(h, m)| h.kind == HarvestableKind::Rock && m.is_some())
        .count()
}

#[test]
fn rocks_along_road_places_minimum_when_missing() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(AssetPlugin::default())
        .add_plugins(ResourcePassesPlugin)
        .add_plugins(RocksAlongRoadPassPlugin)
        .init_state::<GameState>()
        .insert_state(GameState::Playing);

    // Resources required by the pass
    app.world_mut().insert_resource(Tunables::default());
    app.world_mut()
        .insert_resource(RandomizationPolicy::default());
    app.world_mut().insert_resource(simple_straight_road());
    app.world_mut().insert_resource(ChunkAssets::default());

    // Tight config to ensure some placements
    app.world_mut().insert_resource(RocksAlongRoadConfig {
        min_rocks_along_road: 5,
        corridor_half_width: 10.0,
        min_spacing: 1.0,
    });

    app.update();

    let count = count_rule_rocks(app.world_mut());
    assert!(count >= 5);
}

#[test]
fn rocks_along_road_applies_only_once() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(AssetPlugin::default())
        .add_plugins(ResourcePassesPlugin)
        .add_plugins(RocksAlongRoadPassPlugin)
        .init_state::<GameState>()
        .insert_state(GameState::Playing);

    app.world_mut().insert_resource(Tunables::default());
    app.world_mut()
        .insert_resource(RandomizationPolicy::default());
    app.world_mut().insert_resource(simple_straight_road());
    app.world_mut().insert_resource(ChunkAssets::default());
    app.world_mut()
        .insert_resource(RocksAlongRoadConfig::default());

    app.update();
    let first = count_rule_rocks(app.world_mut());
    app.update();
    let second = count_rule_rocks(app.world_mut());
    assert_eq!(first, second, "second run should not place more");
}
