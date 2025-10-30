use crate::components::EnemyKind;
use crate::constants::Tunables;
use crate::waves::rules::{Multipliers, WavePlan, WaveRules};
use bevy::prelude::*;
use bevy::time::TimerMode;
use std::collections::HashMap;
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
    pub current_multipliers: HashMap<EnemyKind, Multipliers>,
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
            current_multipliers: HashMap::new(),
        }
    }

    pub fn start_next_wave(
        &mut self,
        tunables: &Tunables,
        seed_mode: Option<u64>,
        rules: &WaveRules,
    ) {
        self.current_wave += 1;
        self.phase = WavePhase::Spawning;
        // Build from rules
        let plan = rules.plan(self.current_wave, tunables, seed_mode);
        self.spawn_queue.clear();
        for k in plan.enemies.iter().copied() {
            self.spawn_queue.push_back(k);
        }

        self.enemies_to_spawn = self.spawn_queue.len() as u32;
        self.enemies_spawned = 0;
        self.current_multipliers.clear();
        self.current_multipliers
            .extend(plan.multipliers.into_iter());
        self.spawn_timer
            .set_duration(Duration::from_secs_f32(tunables.enemy_spawn_interval_secs));
        self.spawn_timer.reset();
    }

    pub fn start_next_wave_from_plan(&mut self, tunables: &Tunables, plan: WavePlan) {
        self.current_wave += 1;
        self.phase = WavePhase::Spawning;
        self.spawn_queue.clear();
        for k in plan.enemies.iter().copied() {
            self.spawn_queue.push_back(k);
        }
        self.enemies_to_spawn = self.spawn_queue.len() as u32;
        self.enemies_spawned = 0;
        self.current_multipliers.clear();
        self.current_multipliers
            .extend(plan.multipliers.into_iter());
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

    pub fn multiplier_for(&self, kind: EnemyKind) -> Multipliers {
        self.current_multipliers
            .get(&kind)
            .copied()
            .unwrap_or_default()
    }
}
