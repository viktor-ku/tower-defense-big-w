use bevy::prelude::*;

use crate::RoadPaths;
use crate::components::GameState;
use crate::components::harvesting::{Harvestable, HarvestableKind};
use crate::constants::Tunables;
use crate::random_policy::RandomizationPolicy;
use crate::systems::chunks::ChunkAssets;
use crate::systems::resource_passes::{
    PlacedByRule, ResourcePassSet, ResourceRuleConfig, distance_to_polyline_xz,
    sample_point_on_polyline_xz,
};
use rand::{Rng, SeedableRng, rngs::StdRng};

/// Configuration for the rocks-along-road pass.
#[derive(Resource, Debug, Clone, Copy)]
pub struct RocksAlongRoadConfig {
    pub min_rocks_along_road: u32,
    pub corridor_half_width: f32,
    pub min_spacing: f32,
}

impl Default for RocksAlongRoadConfig {
    fn default() -> Self {
        Self {
            min_rocks_along_road: 8,
            corridor_half_width: 12.0,
            min_spacing: 4.0,
        }
    }
}

pub struct RocksAlongRoadPassPlugin;

impl Plugin for RocksAlongRoadPassPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RocksAlongRoadConfig>()
            .init_resource::<RocksAlongRoadState>()
            .init_resource::<ResourceRuleConfig>()
            .add_systems(
                PostUpdate,
                enforce_rocks_along_road
                    .in_set(ResourcePassSet::Apply)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

const RULE_ID_ROCKS_ALONG_ROAD: u64 = 0xC0BB_BA5E_5EED;

#[derive(Resource, Default)]
struct RocksAlongRoadState {
    applied: bool,
}

#[allow(clippy::type_complexity)]
fn enforce_rocks_along_road(
    mut commands: Commands,
    roads: Option<Res<RoadPaths>>,
    assets: Option<Res<ChunkAssets>>,
    cfg: Res<RocksAlongRoadConfig>,
    rule_cfg: Res<ResourceRuleConfig>,
    tunables: Res<Tunables>,
    policy: Res<RandomizationPolicy>,
    rocks_q: Query<(&Transform, &Harvestable, Option<&PlacedByRule>)>,
    mut state: ResMut<RocksAlongRoadState>,
) {
    // Guards
    if !rule_cfg.enabled {
        return;
    }
    if state.applied {
        return;
    }
    let Some(roads) = roads else {
        return;
    };
    let Some(assets) = assets else {
        return;
    };
    if roads.roads.is_empty() {
        return;
    }

    // Use the primary road (0) for now
    let road = &roads.roads[0];
    if road.len() < 2 {
        return;
    }

    // Corridor definition: outside road and within half-width band
    let road_margin = tunables.road_width * 0.5 + 1.0;

    // Collect existing rock positions in corridor and count
    let mut existing_positions: Vec<Vec3> = Vec::new();
    let mut corridor_count: u32 = 0;
    for (tf, harvestable, _marker) in rocks_q.iter() {
        if harvestable.kind != HarvestableKind::Rock {
            continue;
        }
        let d = distance_to_polyline_xz(tf.translation, road);
        if d >= road_margin && d <= cfg.corridor_half_width {
            existing_positions.push(tf.translation);
            corridor_count += 1;
        }
    }

    // Only apply once; if already sufficient, just mark applied and exit
    if corridor_count >= cfg.min_rocks_along_road {
        state.applied = true;
        return;
    }

    // RNG
    let mut rng = if policy.resource_rules_seeded {
        let seed = tunables.world_seed ^ 0xA11C_E55E_D00D ^ RULE_ID_ROCKS_ALONG_ROAD;
        StdRng::seed_from_u64(seed)
    } else {
        // Note: still use StdRng to keep interface consistent; seed with thread RNG
        let s: u64 = rand::rng().random();
        StdRng::seed_from_u64(s)
    };

    let mut to_place = (cfg.min_rocks_along_road - corridor_count) as i32;
    let mut attempts = 0;
    let max_attempts = 200;
    while to_place > 0 && attempts < max_attempts {
        attempts += 1;
        // Sample along the road
        let t = rng.random::<f32>().clamp(0.0, 0.9999);
        let (center, dir) = sample_point_on_polyline_xz(road, t);
        if dir.length_squared() <= f32::EPSILON {
            continue;
        }
        // Pick side and offset distance
        let side = if rng.random::<f32>() < 0.5 { -1.0 } else { 1.0 };
        let offset = rng.random_range(road_margin..=cfg.corridor_half_width);
        let right = Vec3::new(-dir.z, 0.0, dir.x);
        let candidate = center + right * (side * offset);

        // Simple exclusion near the plaza/base center (last road point)
        let plaza = *road.last().unwrap_or(&center);
        let avoid_r = (tunables.plaza_short_side * 0.6).max(10.0);
        if Vec2::new(candidate.x - plaza.x, candidate.z - plaza.z).length() < avoid_r {
            continue;
        }

        // Enforce spacing to existing and newly placed ones
        let too_close = existing_positions
            .iter()
            .any(|p| Vec2::new(candidate.x - p.x, candidate.z - p.z).length() < cfg.min_spacing);
        if too_close {
            continue;
        }

        // Place rock
        let rock_pos = Vec3::new(candidate.x, 0.3, candidate.z);
        commands.spawn((
            Mesh3d(assets.rock_mesh.clone()),
            MeshMaterial3d(assets.rock_mat.clone()),
            Transform::from_translation(rock_pos),
            Harvestable {
                kind: HarvestableKind::Rock,
                amount: 10,
            },
            PlacedByRule {
                id: RULE_ID_ROCKS_ALONG_ROAD,
            },
        ));

        existing_positions.push(rock_pos);
        to_place -= 1;
    }

    // Mark pass as applied regardless of whether we reached the exact target, to avoid
    // continuous replenishment after the player harvests.
    state.applied = true;
}
