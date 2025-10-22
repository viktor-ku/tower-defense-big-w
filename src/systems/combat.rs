use crate::components::{Resource as GameResource, *};
use crate::events::*;
use bevy::prelude::*;

pub fn resource_collection(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    resource_query: Query<(Entity, &Transform, &GameResource)>,
    mut resource_events: MessageWriter<ResourceCollected>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (entity, resource_transform, resource) in resource_query.iter() {
            let distance = player_transform
                .translation
                .distance(resource_transform.translation);

            if distance < 30.0 {
                commands.entity(entity).despawn();
                resource_events.write(ResourceCollected {
                    resource_type: resource.resource_type,
                    amount: resource.amount,
                });
            }
        }
    }
}

pub fn tower_building(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    player_query: Query<&Transform, (With<Player>, Without<Tower>)>,
    building_mode_query: Query<&BuildingMode>,
    mut tower_events: MessageWriter<TowerBuilt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(building_mode) = building_mode_query.single() else {
        return;
    };
    if !building_mode.is_active {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            let Ok(player_transform) = player_query.single() else {
                return;
            };
            let distance = player_transform.translation.distance(Vec3::new(
                world_position.x,
                0.0,
                world_position.y,
            ));

            if distance < 100.0 {
                let tower_position = Vec3::new(world_position.x, 0.0, world_position.y);

                let t_mesh = meshes.add(Cuboid::new(1.2, 2.5, 1.2));
                let t_mat = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.35, 0.35, 0.35),
                    perceptual_roughness: 0.8,
                    metallic: 0.0,
                    ..default()
                });
                commands.spawn((
                    Mesh3d(t_mesh),
                    MeshMaterial3d(t_mat),
                    Transform::from_translation(Vec3::new(
                        tower_position.x,
                        1.25,
                        tower_position.y,
                    )),
                    Tower {
                        range: 80.0,
                        damage: 25,
                        last_shot: 0.0,
                    },
                ));

                tower_events.write(TowerBuilt {
                    position: tower_position,
                });
            }
        }
    }
}

pub fn enemy_spawning(
    mut commands: Commands,
    time: Res<Time>,
    mut enemy_events: MessageWriter<EnemySpawned>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    roads: Option<Res<RoadPaths>>,
) {
    static mut LAST_SPAWN: f32 = 0.0;
    unsafe {
        LAST_SPAWN += time.delta_secs();
        if LAST_SPAWN >= 3.0 {
            LAST_SPAWN = 0.0;

            let (spawn_pos, _road_index_for_follower) = if let Some(roads) = &roads {
                if !roads.roads.is_empty() {
                    let n = roads.roads.len() as f32;
                    let mut ri = (rand::random::<f32>() * n).floor() as usize;
                    if ri >= roads.roads.len() {
                        ri = roads.roads.len() - 1;
                    }
                    let wp = &roads.roads[ri][0];
                    (Vec3::new(wp.x, 0.0, wp.z), Some(ri))
                } else {
                    let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
                    let distance = 200.0;
                    (
                        Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance),
                        None,
                    )
                }
            } else {
                let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
                let distance = 200.0;
                (
                    Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance),
                    None,
                )
            };

            let e_mesh = meshes.add(Cuboid::new(0.9, 1.6, 0.9));
            let e_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.9, 0.1, 0.1),
                perceptual_roughness: 0.7,
                metallic: 0.0,
                ..default()
            });
            // Random speed between 20.0 and 50.0 for variety
            let random_speed = 20.0 + rand::random::<f32>() * 30.0;

            commands.spawn((
                Mesh3d(e_mesh),
                MeshMaterial3d(e_mat),
                Transform::from_translation(Vec3::new(spawn_pos.x, 0.8, spawn_pos.z)),
                Enemy {
                    health: 50,
                    speed: random_speed,
                },
                // Attach PathFollower to follow the chosen road
                match _road_index_for_follower {
                    Some(ri) => PathFollower {
                        road_index: ri,
                        next_index: 1,
                    },
                    None => PathFollower {
                        road_index: 0,
                        next_index: 0,
                    },
                },
            ));
            enemy_events.write(EnemySpawned {
                position: spawn_pos,
            });
        }
    }
}

pub fn tower_shooting(
    time: Res<Time>,
    mut commands: Commands,
    mut tower_query: Query<(&Transform, &mut Tower)>,
    enemy_pos: Query<(&Transform, Entity), With<Enemy>>,
    mut enemy_mut: Query<&mut Enemy>,
    mut enemy_killed_events: MessageWriter<EnemyKilled>,
) {
    for (tower_transform, mut tower) in tower_query.iter_mut() {
        tower.last_shot += time.delta_secs();

        if tower.last_shot >= 1.0 {
            // Find closest enemy in range
            let mut closest_enemy: Option<(Vec3, Entity)> = None;
            let mut closest_distance = f32::MAX;

            for (enemy_transform, entity) in enemy_pos.iter() {
                let distance = tower_transform
                    .translation
                    .distance(enemy_transform.translation);

                if distance <= tower.range && distance < closest_distance {
                    closest_enemy = Some((enemy_transform.translation, entity));
                    closest_distance = distance;
                }
            }

            if let Some((enemy_pos_vec, enemy_entity)) = closest_enemy {
                if let Ok(mut enemy) = enemy_mut.get_mut(enemy_entity) {
                    let new_health = enemy.health.saturating_sub(tower.damage);
                    enemy.health = new_health;
                    tower.last_shot = 0.0;
                    if enemy.health == 0 {
                        // Despawn the enemy entity immediately
                        commands.entity(enemy_entity).despawn();
                        enemy_killed_events.write(EnemyKilled {
                            position: enemy_pos_vec,
                        });
                    }
                }
            }
        }
    }
}
