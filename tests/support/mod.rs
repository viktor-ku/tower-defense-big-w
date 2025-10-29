use bevy::prelude::*;
use std::time::Duration;

use td::components::towers::{BuiltTower, Tower, TowerKind};
use td::components::{Enemy, EnemyKind};
use td::constants::Tunables;
use td::materials::ImpactMaterial;
use td::random_policy::RandomizationPolicy;
use td::systems::combat::assets::CombatVfxAssets;

/// Minimal ECS harness for unit tests.
/// - Inserts `Tunables` with a chosen `world_seed` to drive seeded RNG across systems
/// - Provides a controllable `Time` resource with helpers to advance time deterministically
pub struct TestHarness {
    world: World,
}

impl TestHarness {
    /// Create a new world with deterministic seed and default resources needed by logic tests.
    pub fn new_with_seed(seed: u64) -> Self {
        let mut world = World::new();

        // Tunables with deterministic world seed
        let mut tunables = Tunables::default();
        tunables.world_seed = seed;
        world.insert_resource(tunables);

        // Default policy: everything seeded (can be overridden in tests)
        world.insert_resource(RandomizationPolicy::default());

        // Time resource with zero delta to start
        // Alias `Time` comes from Bevy's prelude (real time). For tests we advance it manually.
        world.insert_resource(Time::<()>::default());

        TestHarness { world }
    }

    /// Mutable access to the underlying `World` for spawning entities/resources.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Immutable access to the underlying `World`.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Set the frame delta for the `Time` resource.
    pub fn set_delta_seconds(&mut self, seconds: f32) {
        let mut time = self
            .world
            .get_resource_mut::<Time<()>>()
            .expect("Time not found");
        // Prefer the modern API when available; fall back if not.
        // Bevy 0.17 provides `advance_by` for time types.
        time.advance_by(Duration::from_secs_f32(seconds));
    }

    /// Advance time by N frames of a fixed delta.
    pub fn advance_frames(&mut self, frames: usize, delta_seconds: f32) {
        for _ in 0..frames {
            self.set_delta_seconds(delta_seconds);
        }
    }
}

/// Ensure core asset stores exist in the world for systems that may touch them.
pub fn ensure_asset_stores(world: &mut World) {
    if world.get_resource::<Assets<Mesh>>().is_none() {
        world.insert_resource(Assets::<Mesh>::default());
    }
    if world.get_resource::<Assets<StandardMaterial>>().is_none() {
        world.insert_resource(Assets::<StandardMaterial>::default());
    }
    if world.get_resource::<Assets<ImpactMaterial>>().is_none() {
        world.insert_resource(Assets::<ImpactMaterial>::default());
    }
    if world.get_resource::<CombatVfxAssets>().is_none() {
        world.insert_resource(CombatVfxAssets::default());
    }
}

/// Spawn a minimal enemy entity with stats derived from its kind.
pub fn spawn_enemy(world: &mut World, kind: EnemyKind, position: Vec3) -> Entity {
    let (hp, dmg, spd, size) = kind.stats();
    world
        .spawn((
            kind,
            Enemy {
                health: hp,
                max_health: hp,
                speed: spd,
                damage: dmg,
            },
            Transform::from_translation(Vec3::new(position.x, size * 0.5, position.z)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .id()
}

/// Spawn a minimal tower entity of a given kind at a position.
pub fn spawn_tower(
    world: &mut World,
    kind: TowerKind,
    position: Vec3,
    tunables: &Tunables,
) -> Entity {
    let (damage, fire_interval_secs, projectile_speed, height) = match kind {
        TowerKind::Bow => (12, 0.7, 60.0, 2.72),
        TowerKind::Crossbow => (35, 2.4, 140.0, 3.68),
    };
    world
        .spawn((
            Tower {
                range: tunables.tower_range,
                damage,
                fire_interval_secs,
                height,
                projectile_speed,
                last_shot: 0.0,
            },
            BuiltTower { kind },
            Transform::from_translation(Vec3::new(position.x, height * 0.5, position.z)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .id()
}

/// Step a custom schedule one frame with a fixed delta, running it on the given world.
pub fn step_schedule(world: &mut World, schedule: &mut Schedule, delta_seconds: f32) {
    // Advance time deterministically for this step
    if world.get_resource::<Time<()>>().is_none() {
        world.insert_resource(Time::<()>::default());
    }
    {
        let mut time = world.get_resource_mut::<Time<()>>().unwrap();
        time.advance_by(Duration::from_secs_f32(delta_seconds));
    }
    schedule.run(world);
}
