use crate::components::EnemyKind;
use crate::constants::Tunables;
use bevy::prelude::*;
use bevy::time::TimerMode;
use std::collections::VecDeque;
use std::time::Duration;

/// Wave phase used by wave progression systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WavePhase {
    Intermission,
    Spawning,
}

/// Global wave state resource tracking timers and counts.
#[derive(Resource, Debug)]
pub struct WaveState {
    pub current_wave: u32,
    pub phase: WavePhase,
    pub intermission_timer: Timer,
    pub spawn_timer: Timer,
    pub enemies_to_spawn: u32,
    pub enemies_spawned: u32,
    pub spawn_queue: VecDeque<EnemyKind>,
}

impl WaveState {
    pub fn new(tunables: &Tunables) -> Self {
        WaveState {
            current_wave: 0,
            phase: WavePhase::Intermission,
            intermission_timer: Timer::from_seconds(
                tunables.wave_initial_delay_secs,
                TimerMode::Once,
            ),
            spawn_timer: Timer::from_seconds(
                tunables.enemy_spawn_interval_secs,
                TimerMode::Repeating,
            ),
            enemies_to_spawn: 0,
            enemies_spawned: 0,
            spawn_queue: VecDeque::new(),
        }
    }

    pub fn start_next_wave(&mut self, tunables: &Tunables) {
        self.current_wave += 1;
        self.phase = WavePhase::Spawning;

        // Build composition for this wave
        let base = self.wave_enemy_count(tunables) as usize;
        // Minions: 50–60%; Zombies: 20–30%; remainder -> minions to keep majority
        let rm = 0.5 + rand::random::<f32>() * 0.1;
        let rz = 0.2 + rand::random::<f32>() * 0.1;
        let mut minions = (rm * base as f32).floor() as usize;
        let zombies = (rz * base as f32).floor() as usize;
        // Ensure remainder goes to minions (keeps them majority)
        minions = base.saturating_sub(zombies).max(minions);

        let mut list: Vec<EnemyKind> = Vec::with_capacity(base);
        list.extend(std::iter::repeat(EnemyKind::Minion).take(minions));
        list.extend(std::iter::repeat(EnemyKind::Zombie).take(base - minions));

        // Shuffle for random mixing
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        list.shuffle(&mut rng);

        // Build spawn queue; boss first on every 10th wave, added on top
        self.spawn_queue.clear();
        if self.current_wave % 10 == 0 {
            self.spawn_queue.push_back(EnemyKind::Boss);
        }
        for k in list {
            self.spawn_queue.push_back(k);
        }

        self.enemies_to_spawn = self.spawn_queue.len() as u32;
        self.enemies_spawned = 0;
        self.spawn_timer
            .set_duration(Duration::from_secs_f32(tunables.enemy_spawn_interval_secs));
        self.spawn_timer.reset();
    }

    pub fn start_intermission(&mut self, duration_secs: f32) {
        self.phase = WavePhase::Intermission;
        self.intermission_timer
            .set_duration(Duration::from_secs_f32(duration_secs));
        self.intermission_timer.reset();
    }

    pub fn upcoming_wave_number(&self) -> u32 {
        match self.phase {
            WavePhase::Intermission => self.current_wave + 1,
            WavePhase::Spawning => self.current_wave.max(1),
        }
    }

    pub fn remaining_intermission_secs(&self) -> f32 {
        self.intermission_timer.remaining_secs()
    }

    fn wave_enemy_count(&self, tunables: &Tunables) -> u32 {
        tunables.wave_base_enemy_count
            + (self.current_wave.saturating_sub(1)) * tunables.wave_enemy_increment
    }
}
