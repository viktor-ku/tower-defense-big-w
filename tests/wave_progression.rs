use bevy::prelude::*;

use bevy::ecs::message::Messages;
use td::audio::{BossWaveStartedEvent, WaveStartedEvent};
use td::components::waves::{WavePhase, WaveState};
use td::constants::Tunables;
use td::random_policy::RandomizationPolicy;
use td::systems::chunks::WorldSeed;
use td::systems::waves::wave_progression;

#[test]
fn wave_progression_transitions_from_initial_delay_to_spawning() {
    let mut world = World::new();
    // Tunables with tiny delays to speed up the test
    let mut t = Tunables::default();
    t.wave_initial_delay_secs = 0.01;
    t.wave_intermission_secs = 0.01;
    world.insert_resource(t.clone());
    world.insert_resource(RandomizationPolicy::default());
    world.insert_resource(WorldSeed(12345));
    world.insert_resource(WaveState::new(&t));
    world.insert_resource(Messages::<WaveStartedEvent>::default());
    world.insert_resource(Messages::<BossWaveStartedEvent>::default());
    world.insert_resource(Time::<()>::default());

    let mut schedule = Schedule::default();
    schedule.add_systems(wave_progression);

    // Advance time beyond initial delay
    for _ in 0..5 {
        world
            .resource_mut::<Time<()>>()
            .advance_by(std::time::Duration::from_millis(5));
        schedule.run(&mut world);
    }

    let s = world.get_resource::<WaveState>().unwrap();
    assert_eq!(s.phase, WavePhase::Spawning);
    assert!(s.enemies_to_spawn > 0);
}

#[test]
fn wave_progression_intermission_after_spawning_when_no_enemies_alive() {
    let mut world = World::new();
    let mut t = Tunables::default();
    t.wave_initial_delay_secs = 0.0;
    t.wave_intermission_secs = 0.01;
    world.insert_resource(t.clone());
    world.insert_resource(RandomizationPolicy::default());
    world.insert_resource(WorldSeed(999));
    let mut s = WaveState::new(&t);
    // Start wave immediately
    s.start_next_wave(&t, Some(999));
    world.insert_resource(s);
    world.insert_resource(Messages::<WaveStartedEvent>::default());
    world.insert_resource(Messages::<BossWaveStartedEvent>::default());
    world.insert_resource(Time::<()>::default());

    let mut schedule = Schedule::default();
    schedule.add_systems(wave_progression);

    // Tick once with no enemies present and all spawned
    {
        let mut ws = world.resource_mut::<WaveState>();
        ws.enemies_spawned = ws.enemies_to_spawn;
    }
    world
        .resource_mut::<Time<()>>()
        .advance_by(std::time::Duration::from_millis(16));
    schedule.run(&mut world);

    let s = world.get_resource::<WaveState>().unwrap();
    assert_eq!(s.phase, WavePhase::Intermission);
}
