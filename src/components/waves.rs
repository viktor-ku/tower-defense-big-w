use crate::constants::Tunables;
use bevy::prelude::*;
use bevy::time::TimerMode;
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
}

impl WaveState {
    pub fn new(tunables: &Tunables) -> Self {
        WaveState {
            current_wave: 0,
            phase: WavePhase::Intermission,
            intermission_timer: Timer::from_seconds(tunables.wave_initial_delay_secs, TimerMode::Once),
            spawn_timer: Timer::from_seconds(tunables.enemy_spawn_interval_secs, TimerMode::Repeating),
            enemies_to_spawn: 0,
            enemies_spawned: 0,
        }
    }

    pub fn start_next_wave(&mut self, tunables: &Tunables) {
        self.current_wave += 1;
        self.phase = WavePhase::Spawning;
        self.enemies_to_spawn = self.wave_enemy_count(tunables);
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


