use super::assets::EnemyHealthBarAssets;
use crate::components::{
    Enemy, EnemyHealthBarFill, EnemyHealthBarRoot, EnemyKind, PathFollower, RoadPaths, WavePhase,
    WaveState,
};
use crate::constants::Tunables;
use crate::events::EnemySpawned;
use bevy::math::primitives::Cuboid;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use std::f32::consts::PI;
use std::time::Duration;

/// Spawns enemies at intervals on road entrances or at a fallback ring.
#[allow(clippy::too_many_arguments)]
pub fn enemy_spawning(
    mut commands: Commands,
    time: Res<Time>,
    mut enemy_events: MessageWriter<EnemySpawned>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut health_bar_assets: ResMut<EnemyHealthBarAssets>,
    roads: Option<Res<RoadPaths>>,
    tunables: Res<Tunables>,
    mut wave_state: ResMut<WaveState>,
) {
    if wave_state.phase != WavePhase::Spawning {
        return;
    }

    if wave_state.enemies_spawned >= wave_state.enemies_to_spawn {
        return;
    }

    if wave_state.spawn_timer.duration()
        != Duration::from_secs_f32(tunables.enemy_spawn_interval_secs)
    {
        wave_state
            .spawn_timer
            .set_duration(Duration::from_secs_f32(tunables.enemy_spawn_interval_secs));
    }

    wave_state.spawn_timer.tick(time.delta());
    if wave_state.spawn_timer.just_finished() {
        let (spawn_pos, road_index) = select_spawn_point(&roads, &tunables);

        // Determine which enemy to spawn next
        if let Some(kind) = wave_state.spawn_queue.pop_front() {
            let (hp, dmg, spd, size) = kind.stats();
            let half_h = size * 0.5;
            let color = match kind {
                EnemyKind::Minion => Color::srgb(0.9, 0.1, 0.1),
                EnemyKind::Zombie => Color::srgb(0.2, 0.8, 0.2),
                EnemyKind::Boss => Color::srgb(0.6, 0.1, 0.8),
            };

            let e_mesh = meshes.add(Cuboid::new(size, size, size));
            let e_mat = materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.7,
                metallic: 0.0,
                ..default()
            });

            let enemy_entity = commands
                .spawn((
                    Mesh3d(e_mesh),
                    MeshMaterial3d(e_mat),
                    Transform::from_translation(Vec3::new(spawn_pos.x, half_h, spawn_pos.z)),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    Enemy {
                        health: hp,
                        max_health: hp,
                        speed: spd,
                        damage: dmg,
                        kind,
                        visual_height: size,
                    },
                    match road_index {
                        Some(ri) => PathFollower {
                            road_index: ri,
                            next_index: 1,
                        },
                        None => PathFollower {
                            road_index: 0,
                            next_index: 0,
                        },
                    },
                ))
                .id();

            attach_health_bar(
                &mut commands,
                enemy_entity,
                &mut meshes,
                &mut materials,
                &mut health_bar_assets,
                &tunables,
                size,
            );

            enemy_events.write(EnemySpawned {
                position: spawn_pos,
            });
            wave_state.enemies_spawned += 1;
        }
    }
}

fn select_spawn_point(
    roads: &Option<Res<RoadPaths>>,
    tunables: &Tunables,
) -> (Vec3, Option<usize>) {
    if let Some(roads) = roads
        && !roads.roads.is_empty()
    {
        let mut ri = (rand::random::<f32>() * roads.roads.len() as f32).floor() as usize;
        if ri >= roads.roads.len() {
            ri = roads.roads.len() - 1;
        }
        let wp = &roads.roads[ri][0];
        return (Vec3::new(wp.x, 0.0, wp.z), Some(ri));
    }

    let angle = rand::random::<f32>() * 2.0 * PI;
    let distance = tunables.enemy_spawn_ring_distance;
    (
        Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance),
        None,
    )
}

fn attach_health_bar(
    commands: &mut Commands,
    enemy_entity: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    health_bar_assets: &mut ResMut<EnemyHealthBarAssets>,
    tunables: &Tunables,
    base_height: f32,
) {
    let quad_mesh = health_bar_assets.mesh(meshes);
    let background_mat = health_bar_assets.background_material(materials);
    let fill_mat = health_bar_assets.fill_material(materials);

    commands.entity(enemy_entity).with_children(|parent| {
        parent
            .spawn((
                EnemyHealthBarRoot,
                Transform::from_translation(Vec3::new(
                    0.0,
                    base_height + tunables.health_bar_offset_y,
                    0.0,
                )),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
            ))
            .with_children(|bar| {
                bar.spawn((
                    Mesh3d(quad_mesh.clone()),
                    MeshMaterial3d(background_mat.clone()),
                    Transform {
                        translation: Vec3::ZERO,
                        scale: Vec3::new(
                            tunables.health_bar_width,
                            tunables.health_bar_height,
                            1.0,
                        ),
                        ..default()
                    },
                ));

                bar.spawn((
                    Mesh3d(quad_mesh.clone()),
                    MeshMaterial3d(fill_mat),
                    Transform {
                        translation: Vec3::new(0.0, 0.0, 0.001),
                        scale: Vec3::new(
                            tunables.health_bar_width,
                            tunables.health_bar_fill_height,
                            1.0,
                        ),
                        ..default()
                    },
                    EnemyHealthBarFill {
                        max_width: tunables.health_bar_width,
                        owner: enemy_entity,
                        last_ratio: 1.0,
                    },
                ));
            });
    });
}

pub fn update_enemy_health_bars(
    enemy_query: Query<&Enemy>,
    mut fill_query: Query<(&mut EnemyHealthBarFill, &mut Transform)>,
) {
    for (mut fill, mut transform) in fill_query.iter_mut() {
        if let Ok(enemy) = enemy_query.get(fill.owner) {
            let ratio = if enemy.max_health > 0 {
                enemy.health as f32 / enemy.max_health as f32
            } else {
                0.0
            }
            .clamp(0.0, 1.0);

            if (ratio - fill.last_ratio).abs() > 0.001 {
                fill.last_ratio = ratio;
                let width = fill.max_width * ratio;
                transform.scale.x = width.max(0.0);
                transform.translation.x = -fill.max_width * 0.5 + width * 0.5;
            }
        }
    }
}

pub fn face_enemy_health_bars(
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    mut bars: Query<(&mut Transform, &GlobalTransform), With<EnemyHealthBarRoot>>,
) {
    let Ok(camera_tf) = camera_query.single() else {
        return;
    };
    let cam_pos = camera_tf.translation();

    for (mut transform, global) in bars.iter_mut() {
        let bar_pos = global.translation();
        let dir = Vec3::new(cam_pos.x - bar_pos.x, 0.0, cam_pos.z - bar_pos.z);
        if dir.length_squared() > f32::EPSILON {
            let yaw = dir.x.atan2(dir.z);
            transform.rotation = Quat::from_rotation_y(yaw);
        }
    }
}
