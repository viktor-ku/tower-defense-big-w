use bevy::prelude::*;

use td::components::enemies::EnemyKind;
use td::components::waves::{WavePhase, WaveState};
use td::constants::Tunables;

fn collect_queue(state: &WaveState) -> Vec<EnemyKind> {
    state.spawn_queue.iter().copied().collect()
}

#[test]
fn wave_enemy_count_and_boss_insertion() {
    let tunables = Tunables::default();

    // Wave 1
    let mut s = WaveState::new(&tunables);
    s.start_next_wave(&tunables, Some(tunables.world_seed));
    let base1 = tunables.wave_base_enemy_count;
    assert_eq!(s.phase, WavePhase::Spawning);
    assert_eq!(s.current_wave, 1);
    assert_eq!(s.enemies_to_spawn, base1);

    // Advance to wave 10
    for _ in 0..8 {
        s.start_next_wave(&tunables, Some(tunables.world_seed));
    }
    // Now at wave 10
    s.start_next_wave(&tunables, Some(tunables.world_seed));
    assert_eq!(s.current_wave, 10);
    let base10 = tunables.wave_base_enemy_count + (9) * tunables.wave_enemy_increment;
    // Boss added on top
    assert_eq!(s.enemies_to_spawn, base10 + 1);
}

#[test]
fn wave_queue_is_deterministic_with_same_seed() {
    let tunables = Tunables::default();
    let seed = tunables.world_seed;

    let mut a = WaveState::new(&tunables);
    let mut b = WaveState::new(&tunables);

    a.start_next_wave(&tunables, Some(seed));
    b.start_next_wave(&tunables, Some(seed));

    let qa = collect_queue(&a);
    let qb = collect_queue(&b);
    assert_eq!(qa, qb);
}

#[test]
fn boss_is_first_in_wave_10() {
    let tunables = Tunables::default();
    let mut s = WaveState::new(&tunables);
    for _ in 0..10 {
        s.start_next_wave(&tunables, Some(tunables.world_seed));
    }
    assert_eq!(s.current_wave, 10);
    assert_eq!(s.spawn_queue.front().copied(), Some(EnemyKind::Boss));
}

#[test]
fn wave_queue_differs_with_different_seeds() {
    let tunables = Tunables::default();
    let seed_a = tunables.world_seed;
    let seed_b = tunables.world_seed ^ 0xDEADBEEFDEADBEEF;

    let mut a = WaveState::new(&tunables);
    let mut b = WaveState::new(&tunables);

    a.start_next_wave(&tunables, Some(seed_a));
    b.start_next_wave(&tunables, Some(seed_b));

    let qa = collect_queue(&a);
    let qb = collect_queue(&b);
    assert_ne!(qa, qb, "Different seeds should shuffle differently");
}

#[test]
fn upcoming_wave_number_and_intermission_timer() {
    let tunables = Tunables::default();

    let mut s = WaveState::new(&tunables);
    assert_eq!(s.phase, WavePhase::Intermission);
    assert_eq!(s.upcoming_wave_number(), 1);

    s.start_next_wave(&tunables, Some(tunables.world_seed));
    assert_eq!(s.upcoming_wave_number(), 1);

    s.start_intermission(3.2);
    assert!((s.remaining_intermission_secs() - 3.2).abs() < 1e-5);
}
