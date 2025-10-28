use crate::components::{Enemy, WavePhase, WaveState};
use crate::constants::Tunables;
use crate::random_policy::RandomizationPolicy;
use crate::systems::WorldSeed;
use bevy::audio::{AudioPlayer, PlaybackMode, PlaybackSettings, Volume};
use bevy::prelude::*;
use std::time::Duration;

/// Handles transitioning between wave intermissions and active waves.
pub fn wave_progression(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut wave_state: ResMut<WaveState>,
    tunables: Res<Tunables>,
    enemy_query: Query<Entity, With<Enemy>>,
    seed: Res<WorldSeed>,
    policy: Res<RandomizationPolicy>,
) {
    match wave_state.phase {
        WavePhase::Intermission => {
            let target_duration = if wave_state.current_wave == 0 {
                tunables.wave_initial_delay_secs
            } else {
                tunables.wave_intermission_secs
            };

            if wave_state.intermission_timer.duration() != Duration::from_secs_f32(target_duration)
            {
                wave_state
                    .intermission_timer
                    .set_duration(Duration::from_secs_f32(target_duration));
            }

            wave_state.intermission_timer.tick(time.delta());
            if wave_state.intermission_timer.just_finished() {
                let rel_path = "sounds/round-start.wav";
                let full_path = std::path::Path::new("assets").join(rel_path);
                if full_path.exists() {
                    commands.spawn((
                        AudioPlayer::new(asset_server.load(rel_path)),
                        PlaybackSettings {
                            mode: PlaybackMode::Despawn,
                            volume: Volume::Decibels(-10.0),
                            ..default()
                        },
                    ));
                } else {
                    error!(
                        "Missing sound asset: {}. Skipping playback.",
                        full_path.display()
                    );
                }
                let seed_mode = if policy.wave_composition_seeded {
                    Some(seed.0)
                } else {
                    None
                };
                wave_state.start_next_wave(&tunables, seed_mode);
            }
        }
        WavePhase::Spawning => {
            let no_enemies_alive = enemy_query.iter().next().is_none();
            if wave_state.enemies_spawned >= wave_state.enemies_to_spawn && no_enemies_alive {
                wave_state.start_intermission(tunables.wave_intermission_secs);
            }
        }
    }
}
