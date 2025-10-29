use bevy::prelude::*;

use crate::components::harvesting::Harvestable;
use crate::components::roads::RoadPaths;
use crate::components::town::TownSquareCenter;
use crate::constants::Tunables;
use crate::systems::resource_passes::ResourcePassSet;

pub struct TownSquareExclusionPassPlugin;

impl Plugin for TownSquareExclusionPassPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TownSquareExclusionState>().add_systems(
            PostUpdate,
            enforce_town_square_exclusion
                .in_set(ResourcePassSet::Apply)
                .run_if(resource_exists::<Tunables>),
        );
    }
}

#[derive(Resource, Default)]
struct TownSquareExclusionState {
    applied_once: bool,
}

#[allow(clippy::type_complexity)]
fn enforce_town_square_exclusion(
    mut commands: Commands,
    tunables: Res<Tunables>,
    square_center: Option<Res<TownSquareCenter>>,
    roads: Option<Res<RoadPaths>>,
    mut state: ResMut<TownSquareExclusionState>,
    harvestables_q: Query<(Entity, &Transform), With<Harvestable>>,
) {
    // Apply only once after the square center is known to clean up any early spawns
    if state.applied_once {
        return;
    }

    let center = if let Some(c) = square_center {
        c.0
    } else if let Some(r) = roads {
        // Fallback: use end of the primary road as plaza center
        r.roads
            .get(0)
            .and_then(|road| road.last().copied())
            .unwrap_or(Vec3::ZERO)
    } else {
        return; // No information yet
    };

    let radius = tunables.town_resource_exclusion_radius;

    for (entity, tf) in harvestables_q.iter() {
        let d = Vec2::new(tf.translation.x - center.x, tf.translation.z - center.z).length();
        if d <= radius {
            commands.entity(entity).despawn();
        }
    }

    state.applied_once = true;
}
