use crate::components::*;
use crate::constants::Tunables;
use crate::events::*;
use bevy::asset::RenderAssetUsages;
use bevy::math::primitives::Rectangle;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::time::TimerMode;
use std::f32::consts::TAU;
use std::time::Duration;

// constants moved to Tunables

#[derive(Resource, Default)]
pub struct EnemyHealthBarAssets {
    quad_mesh: Option<Handle<Mesh>>,
    background_material: Option<Handle<StandardMaterial>>,
    fill_material: Option<Handle<StandardMaterial>>,
}

impl EnemyHealthBarAssets {
    fn mesh(&mut self, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.quad_mesh
            .get_or_insert_with(|| meshes.add(build_quad_mesh()))
            .clone()
    }

    fn background_material(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        self.background_material
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgba(0.05, 0.05, 0.05, 0.7),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })
            })
            .clone()
    }

    fn fill_material(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        self.fill_material
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgba(0.2, 0.85, 0.2, 0.9),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })
            })
            .clone()
    }
}
// moved to Tunables

fn cursor_to_ground(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_position: Vec2,
    ground_y: f32,
) -> Option<Vec3> {
    let ray = camera
        .viewport_to_world(camera_transform, cursor_position)
        .ok()?;
    let denom = ray.direction.y;
    if denom.abs() < f32::EPSILON {
        return None;
    }
    let t = (ground_y - ray.origin.y) / denom;
    if t < 0.0 {
        return None;
    }
    let mut point = ray.origin + ray.direction * t;
    point.y = ground_y;
    Some(point)
}

/// Places a tower at the mouse cursor when in building mode and within range.
pub fn tower_building(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut transforms: ParamSet<(
        Query<&Transform, (With<Player>, Without<Tower>)>,
        Query<&mut Transform, With<TowerGhost>>,
    )>,
    mut player_res_query: Query<&mut Player, With<Player>>,
    building_mode_query: Query<&BuildingMode>,
    mut tower_events: MessageWriter<TowerBuilt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ghost_state: Local<Option<TowerGhostData>>,
    tunables: Res<Tunables>,
) {
    let building_mode_active = building_mode_query.iter().any(|mode| mode.is_active);

    if !building_mode_active {
        clear_ghost(&mut commands, &mut meshes, &mut materials, &mut ghost_state);
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        clear_ghost(&mut commands, &mut meshes, &mut materials, &mut ghost_state);
        return;
    };
    let Some(world_point) = cursor_to_ground(&camera, &camera_transform, cursor_position, 0.0)
    else {
        clear_ghost(&mut commands, &mut meshes, &mut materials, &mut ghost_state);
        return;
    };

    let player_query = transforms.p0();
    let Some(player_transform) = player_query.iter().next() else {
        clear_ghost(&mut commands, &mut meshes, &mut materials, &mut ghost_state);
        return;
    };

    let player_pos = Vec3::new(
        player_transform.translation.x,
        0.0,
        player_transform.translation.z,
    );
    let mut offset = world_point - player_pos;
    offset.y = 0.0;
    let distance_sq = offset.length_squared();
    let max_build_distance_sq = tunables.max_build_distance * tunables.max_build_distance;
    let in_range = distance_sq <= max_build_distance_sq;
    if distance_sq > max_build_distance_sq && distance_sq > 0.0 {
        offset = offset.normalize() * tunables.max_build_distance;
    }
    let placement_pos = player_pos + offset;

    // Spawn or update ghost preview
    let state = ghost_state.get_or_insert_with(|| {
        spawn_tower_ghost(&mut commands, &mut meshes, &mut materials, &tunables)
    });

    let mut ghost_query = transforms.p1();
    if let Ok(mut transform) = ghost_query.get_mut(state.root) {
        transform.translation = placement_pos;
    }
    // Check affordability
    let mut affordable = false;
    if let Ok(player) = player_res_query.single_mut() {
        affordable =
            player.wood >= tunables.tower_cost_wood && player.rock >= tunables.tower_cost_rock;
    }

    update_ghost_visuals(state, in_range && affordable, &mut materials);

    if in_range && affordable && mouse_input.just_pressed(MouseButton::Left) {
        if let Ok(mut player) = player_res_query.single_mut() {
            // Deduct resources
            player.wood = player.wood.saturating_sub(tunables.tower_cost_wood);
            player.rock = player.rock.saturating_sub(tunables.tower_cost_rock);
        }
        place_tower(
            &mut commands,
            &mut meshes,
            &mut materials,
            placement_pos,
            &mut tower_events,
            &tunables,
        );
    }
}

pub struct TowerGhostData {
    root: Entity,
    tower_child: Entity,
    range_child: Entity,
    tower_material: Handle<StandardMaterial>,
    ring_material: Handle<StandardMaterial>,
    ring_mesh: Handle<Mesh>,
}

fn spawn_tower_ghost(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tunables: &Tunables,
) -> TowerGhostData {
    let tower_mesh = meshes.add(Cuboid::new(
        tunables.tower_width,
        tunables.tower_height,
        tunables.tower_depth,
    ));
    let range_mesh = meshes.add(build_ring_mesh(
        tunables.tower_range,
        tunables.ring_inner_ratio,
        96,
    ));

    let tower_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.35, 0.35, 0.35, 0.4),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let ring_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.85, 0.2, 0.35),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let root = commands
        .spawn((TowerGhost, Transform::default(), GlobalTransform::default()))
        .id();

    let mut tower_child = None;
    let mut range_child = None;

    commands.entity(root).with_children(|parent| {
        tower_child = Some(
            parent
                .spawn((
                    Mesh3d(tower_mesh.clone()),
                    MeshMaterial3d(tower_material.clone()),
                    Transform::from_translation(Vec3::new(0.0, tunables.tower_height * 0.5, 0.0)),
                    GlobalTransform::default(),
                ))
                .id(),
        );
        range_child = Some(
            parent
                .spawn((
                    Mesh3d(range_mesh.clone()),
                    MeshMaterial3d(ring_material.clone()),
                    Transform::from_translation(Vec3::new(0.0, 0.05, 0.0)),
                    GlobalTransform::default(),
                    Visibility::default(),
                ))
                .id(),
        );
    });

    TowerGhostData {
        root,
        tower_child: tower_child.expect("tower ghost mesh child"),
        range_child: range_child.expect("tower ghost range child"),
        tower_material,
        ring_material,
        ring_mesh: range_mesh,
    }
}

fn build_ring_mesh(outer_radius: f32, inner_ratio: f32, segments: usize) -> Mesh {
    let inner_radius = outer_radius * inner_ratio.clamp(0.0, 0.999);

    let mut positions = Vec::with_capacity(segments * 6);
    let mut normals = Vec::with_capacity(segments * 6);
    let mut uvs = Vec::with_capacity(segments * 6);

    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * TAU;
        let next_angle = ((i + 1) as f32 / segments as f32) * TAU;

        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let cos_b = next_angle.cos();
        let sin_b = next_angle.sin();

        let outer_a = Vec3::new(outer_radius * cos_a, 0.0, outer_radius * sin_a);
        let inner_a = Vec3::new(inner_radius * cos_a, 0.0, inner_radius * sin_a);
        let outer_b = Vec3::new(outer_radius * cos_b, 0.0, outer_radius * sin_b);
        let inner_b = Vec3::new(inner_radius * cos_b, 0.0, inner_radius * sin_b);

        // triangle 1
        positions.push([outer_a.x, outer_a.y, outer_a.z]);
        positions.push([inner_a.x, inner_a.y, inner_a.z]);
        positions.push([outer_b.x, outer_b.y, outer_b.z]);

        // triangle 2
        positions.push([outer_b.x, outer_b.y, outer_b.z]);
        positions.push([inner_a.x, inner_a.y, inner_a.z]);
        positions.push([inner_b.x, inner_b.y, inner_b.z]);

        normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 6]);

        let uv_outer_a = [0.5 + 0.5 * cos_a, 0.5 + 0.5 * sin_a];
        let uv_inner_a = [
            0.5 + 0.5 * inner_ratio * cos_a,
            0.5 + 0.5 * inner_ratio * sin_a,
        ];
        let uv_outer_b = [0.5 + 0.5 * cos_b, 0.5 + 0.5 * sin_b];
        let uv_inner_b = [
            0.5 + 0.5 * inner_ratio * cos_b,
            0.5 + 0.5 * inner_ratio * sin_b,
        ];

        uvs.push(uv_outer_a);
        uvs.push(uv_inner_a);
        uvs.push(uv_outer_b);

        uvs.push(uv_outer_b);
        uvs.push(uv_inner_a);
        uvs.push(uv_inner_b);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

fn build_quad_mesh() -> Mesh {
    Mesh::from(Rectangle::new(1.0, 1.0))
}

fn update_ghost_visuals(
    data: &TowerGhostData,
    valid: bool,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let (tower_color, ring_color) = if valid {
        (
            Color::srgba(0.2, 0.85, 0.2, 0.4),
            Color::srgba(0.2, 0.85, 0.2, 0.35),
        )
    } else {
        (
            Color::srgba(0.85, 0.2, 0.2, 0.4),
            Color::srgba(0.85, 0.2, 0.2, 0.35),
        )
    };

    if let Some(material) = materials.get_mut(&data.tower_material) {
        material.base_color = tower_color;
    }
    if let Some(material) = materials.get_mut(&data.ring_material) {
        material.base_color = ring_color;
    }
}

fn place_tower(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    tower_events: &mut MessageWriter<TowerBuilt>,
    tunables: &Tunables,
) {
    let mesh = meshes.add(Cuboid::new(
        tunables.tower_width,
        tunables.tower_height,
        tunables.tower_depth,
    ));
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.35, 0.35),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_translation(Vec3::new(
            position.x,
            tunables.tower_height * 0.5,
            position.z,
        )),
        Tower {
            range: tunables.tower_range,
            damage: tunables.tower_damage,
            last_shot: 0.0,
        },
    ));

    tower_events.write(TowerBuilt { position });

    spawn_tower_spawn_effect(commands, meshes, materials, position, tunables);
}

fn clear_ghost(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    ghost_state: &mut Local<Option<TowerGhostData>>,
) {
    if let Some(data) = ghost_state.take() {
        commands.entity(data.tower_child).despawn();
        commands.entity(data.range_child).despawn();
        commands.entity(data.root).despawn();
        materials.remove(&data.tower_material);
        materials.remove(&data.ring_material);
        meshes.remove(&data.ring_mesh);
    }
}

#[derive(Component)]
pub(crate) struct TowerSpawnEffect {
    timer: Timer,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    base_rgb: [f32; 3],
}

fn spawn_tower_spawn_effect(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    tunables: &Tunables,
) {
    let mesh_handle = meshes.add(build_ring_mesh(
        tunables.tower_range,
        tunables.ring_inner_ratio,
        72,
    ));
    let base_color = [0.9, 0.95, 0.6];
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(base_color[0], base_color[1], base_color[2], 0.7),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(material.clone()),
        Transform {
            translation: Vec3::new(position.x, 0.05, position.z),
            scale: Vec3::splat(0.3),
            ..default()
        },
        GlobalTransform::default(),
        Visibility::default(),
        TowerSpawnEffect {
            timer: Timer::from_seconds(tunables.tower_spawn_effect_duration_secs, TimerMode::Once),
            material,
            mesh: mesh_handle,
            base_rgb: base_color,
        },
    ));
}

pub fn tower_spawn_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut TowerSpawnEffect, &mut Transform)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut effect, mut transform) in effects.iter_mut() {
        effect.timer.tick(time.delta());
        let duration = effect.timer.duration().as_secs_f32().max(f32::EPSILON);
        let elapsed = effect.timer.elapsed().as_secs_f32();
        let t = (elapsed / duration).clamp(0.0, 1.0);
        let eased = t * t;
        transform.scale = Vec3::splat(0.3 + eased * 0.9);

        if let Some(mat) = materials.get_mut(&effect.material) {
            let alpha = (1.0 - t).max(0.0) * 0.7;
            mat.base_color = Color::srgba(
                effect.base_rgb[0],
                effect.base_rgb[1],
                effect.base_rgb[2],
                alpha,
            );
        }

        if effect.timer.is_finished() {
            let material_handle = effect.material.clone();
            commands.entity(entity).despawn();
            materials.remove(&material_handle);
            meshes.remove(&effect.mesh);
        }
    }
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
                // height will be set at spawn time; keep Y scale constant here
                transform.scale = Vec3::new(width.max(0.0), transform.scale.y, 1.0);
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

/// Spawns enemies at intervals on road entrances or at a ring around the map.
pub fn enemy_spawning(
    mut commands: Commands,
    time: Res<Time>,
    mut enemy_events: MessageWriter<EnemySpawned>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut health_bar_assets: ResMut<EnemyHealthBarAssets>,
    roads: Option<Res<RoadPaths>>,
    tunables: Res<Tunables>,
    mut spawn_timer: Local<Option<Timer>>,
) {
    let timer = spawn_timer.get_or_insert_with(|| {
        Timer::from_seconds(tunables.enemy_spawn_interval_secs, TimerMode::Repeating)
    });

    // Keep timer duration synced with tunables
    if timer.duration() != Duration::from_secs_f32(tunables.enemy_spawn_interval_secs) {
        timer.set_duration(Duration::from_secs_f32(tunables.enemy_spawn_interval_secs));
    }

    timer.tick(time.delta());
    if timer.just_finished() {

            let (spawn_pos, _road_index_for_follower) = if let Some(roads) = &roads {
                if !roads.roads.is_empty() {
                    // Pick any road (all roads now go directly to village center)
                    let n = roads.roads.len() as f32;
                    let mut ri = (rand::random::<f32>() * n).floor() as usize;
                    if ri >= roads.roads.len() {
                        ri = roads.roads.len() - 1;
                    }
                    let wp = &roads.roads[ri][0];
                    (Vec3::new(wp.x, 0.0, wp.z), Some(ri))
                } else {
                    let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
                    let distance = tunables.enemy_spawn_ring_distance;
                    (
                        Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance),
                        None,
                    )
                }
            } else {
                let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
                let distance = tunables.enemy_spawn_ring_distance;
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
            // Random speed in configured range
            let random_speed = tunables.enemy_random_speed_min
                + rand::random::<f32>()
                    * (tunables.enemy_random_speed_max - tunables.enemy_random_speed_min);

            let enemy_entity = commands
                .spawn((
                    Mesh3d(e_mesh),
                    MeshMaterial3d(e_mat),
                    Transform::from_translation(Vec3::new(spawn_pos.x, 0.8, spawn_pos.z)),
                    Enemy {
                        health: tunables.enemy_default_health,
                        max_health: tunables.enemy_default_health,
                        speed: random_speed,
                    },
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
                ))
                .id();

            let quad_mesh = health_bar_assets.mesh(&mut meshes);
            let background_mat = health_bar_assets.background_material(&mut materials);
            let fill_mat = health_bar_assets.fill_material(&mut materials);

            commands.entity(enemy_entity).with_children(|parent| {
                parent
                    .spawn((
                        EnemyHealthBarRoot,
                        Transform::from_translation(Vec3::new(
                            0.0,
                            tunables.health_bar_offset_y,
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

            enemy_events.write(EnemySpawned { position: spawn_pos });
    }
}

/// Makes towers shoot the closest enemy in range at a fixed fire rate.
pub fn tower_shooting(
    time: Res<Time>,
    mut commands: Commands,
    mut tower_query: Query<(&Transform, &mut Tower)>,
    enemy_pos: Query<(&Transform, Entity), With<Enemy>>,
    mut enemy_mut: Query<&mut Enemy>,
    mut enemy_killed_events: MessageWriter<EnemyKilled>,
    tunables: Res<Tunables>,
) {
    for (tower_transform, mut tower) in tower_query.iter_mut() {
        tower.last_shot += time.delta_secs();

        if tower.last_shot >= tunables.tower_fire_interval_secs {
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
