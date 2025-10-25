use crate::constants::Tunables;
use bevy::prelude::*;
use bevy::time::TimerMode;
use std::time::Duration;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
}

#[derive(Component)]
pub struct Player {
    pub wood: u32,
    pub rock: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarvestableKind {
    Wood,
    Rock,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Harvestable {
    pub kind: HarvestableKind,
    pub amount: u32,
}

#[derive(Component)]
pub struct Tree;

#[derive(Component)]
pub struct Tower {
    pub range: f32,
    pub damage: u32,
    pub last_shot: f32,
}

/// Marker for the in-progress tower preview (ghost).
#[derive(Component)]
pub struct TowerGhost;

#[derive(Component)]
pub struct Enemy {
    pub health: u32,
    pub max_health: u32,
    pub speed: f32,
}

#[derive(Component)]
pub struct EnemyHealthBarRoot;

#[derive(Component)]
pub struct EnemyHealthBarFill {
    pub max_width: f32,
    pub owner: Entity,
    pub last_ratio: f32,
}

#[derive(Component)]
pub struct Village {
    pub health: u32,
    pub max_health: u32,
}

// DayNight removed

#[derive(Component)]
pub struct BuildingMode {
    pub is_active: bool,
}

// 3D isometric markers
#[derive(Component)]
pub struct IsoPlayer;

// ResourceType removed in favor of HarvestableKind

// Road pathing
#[derive(Resource, Default, Debug, Clone)]
pub struct RoadPaths {
    pub roads: Vec<Vec<Vec3>>, // Each road is a sequence of waypoints (centerline on XZ)
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PathFollower {
    pub road_index: usize,
    pub next_index: usize,
}

/// Tracks the current hold-to-collect target and normalized progress [0,1].
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct CurrentCollectProgress {
    pub target: Option<Entity>,
    pub progress: f32,
}

// Town markers
#[derive(Component)]
pub struct TownCenter;

#[derive(Component)]
pub struct TownSquare;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Gate;

// Chunking: marker for a chunk root entity
#[derive(Component, Debug, Clone, Copy)]
pub struct ChunkRoot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WavePhase {
    Intermission,
    Spawning,
}

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
