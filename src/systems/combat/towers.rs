use crate::audio::{BuildingActionEvent, BuildingActionKind};
use crate::components::{
    BuildingMode, BuiltTower, HasTowerDamageLabel, Player, SellingMode, Tower, TowerBuildSelection,
    TowerDamageLabel, TowerGhost, TowerKind, TowerUpgradeConfig, TowerUpgrades, UpgradeableStat,
};
use crate::constants::Tunables;
use crate::events::TowerBuilt;
use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::MouseButton;
use bevy::math::primitives::Cuboid;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use std::f32::consts::TAU;

/// Places a tower at the cursor when in building mode and within range.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
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
    mut selection: ResMut<TowerBuildSelection>,
    tunables: Res<Tunables>,
    mut building_sfx: MessageWriter<BuildingActionEvent>,
    upgrades: Res<TowerUpgrades>,
    upgrade_config: Res<TowerUpgradeConfig>,
) {
    let building_mode_active = building_mode_query.iter().any(|mode| mode.is_active);

    if !building_mode_active {
        clear_ghost(&mut commands, &mut meshes, &mut materials, &mut ghost_state);
        return;
    }

    // Allow preview if a selection is chosen.
    let preview_kind = selection.choice;
    if preview_kind.is_none() {
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
    let Some(world_point) = cursor_to_ground(camera, camera_transform, cursor_position, 0.0) else {
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

    // Determine preview size from selected kind
    let preview_size: (f32, f32, f32) = match preview_kind.unwrap_or(TowerKind::Bow) {
        // Bow: smaller (absolute size)
        TowerKind::Bow => (1.02, 2.72, 1.02),
        // Crossbow: bigger (absolute size)
        TowerKind::Crossbow => (1.38, 3.68, 1.38),
    };

    // Spawn or update ghost preview
    let state = ghost_state.get_or_insert_with(|| {
        spawn_tower_ghost(
            &mut commands,
            &mut meshes,
            &mut materials,
            &tunables,
            preview_size,
        )
    });

    let mut ghost_query = transforms.p1();
    if let Ok(mut transform) = ghost_query.get_mut(state.root) {
        transform.translation = placement_pos;
    }

    // Check affordability per selected tower kind (centralized costs)
    let mut affordable = false;
    let (wood_cost, rock_cost) = preview_kind.unwrap_or(TowerKind::Bow).cost();
    if let Ok(player) = player_res_query.single_mut() {
        affordable = player.wood >= wood_cost && player.rock >= rock_cost;
    }

    update_ghost_visuals(state, in_range && affordable, &mut materials);

    if in_range
        && affordable
        && mouse_input.just_pressed(MouseButton::Left)
        && selection.choice.is_some()
    {
        let kind = selection.choice.unwrap_or(TowerKind::Bow);
        let (wood_cost, rock_cost) = kind.cost();
        if let Ok(mut player) = player_res_query.single_mut() {
            // Deduct resources based on selected kind
            player.wood = player.wood.saturating_sub(wood_cost);
            player.rock = player.rock.saturating_sub(rock_cost);
        }
        // Determine tower stats from selected kind
        let (base_damage, base_fire_interval, base_projectile_speed, size, color) = match kind {
            // Bow: smaller and blue (absolute size); slower projectiles
            TowerKind::Bow => (
                12,
                0.7,
                60.0,
                (1.02, 2.72, 1.02),
                Color::srgb(0.35, 0.45, 0.95),
            ),
            // Crossbow: bigger and purple (absolute size); much faster projectiles
            TowerKind::Crossbow => (
                35,
                2.4,
                140.0,
                (1.38, 3.68, 1.38),
                Color::srgb(0.62, 0.36, 0.86),
            ),
        };

        // Apply upgrades using declarative config system
        let level = upgrades.get_level(kind);
        let damage_bonus =
            upgrade_config.calculate_bonus(kind, UpgradeableStat::Damage, level) as u32;
        let range_bonus = upgrade_config.calculate_bonus(kind, UpgradeableStat::Range, level);
        let fire_speed_bonus =
            upgrade_config.calculate_bonus(kind, UpgradeableStat::FireSpeed, level);
        let projectile_speed_bonus =
            upgrade_config.calculate_bonus(kind, UpgradeableStat::ProjectileSpeed, level);

        let damage = base_damage + damage_bonus;
        let fire_interval_secs = (base_fire_interval - fire_speed_bonus).max(0.1);
        let projectile_speed = base_projectile_speed + projectile_speed_bonus;
        let range = tunables.tower_range + range_bonus;

        place_tower(
            &mut commands,
            &mut meshes,
            &mut materials,
            placement_pos,
            &mut tower_events,
            damage,
            fire_interval_secs,
            projectile_speed,
            range,
            size,
            color,
            &tunables,
            kind,
        );

        // Emit building place SFX event
        building_sfx.write(BuildingActionEvent {
            kind: BuildingActionKind::Place,
            position: placement_pos,
        });

        // Force re-choose next time
        selection.choice = None;
        clear_ghost(&mut commands, &mut meshes, &mut materials, &mut ghost_state);
    } else if mouse_input.just_pressed(MouseButton::Left) && selection.choice.is_some() {
        // Invalid placement attempt: out of range or not affordable
        building_sfx.write(BuildingActionEvent {
            kind: BuildingActionKind::Invalid,
            position: placement_pos,
        });
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

fn spawn_tower_ghost(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tunables: &Tunables,
    size: (f32, f32, f32),
) -> TowerGhostData {
    let tower_mesh = meshes.add(Cuboid::new(size.0, size.1, size.2));
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
        .spawn((
            TowerGhost,
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .id();

    let mut tower_child = None;
    let mut range_child = None;

    commands.entity(root).with_children(|parent| {
        tower_child = Some(
            parent
                .spawn((
                    Mesh3d(tower_mesh.clone()),
                    MeshMaterial3d(tower_material.clone()),
                    Transform::from_translation(Vec3::new(0.0, size.1 * 0.5, 0.0)),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
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
                    InheritedVisibility::default(),
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

#[allow(clippy::too_many_arguments)]
fn place_tower(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    tower_events: &mut MessageWriter<TowerBuilt>,
    damage: u32,
    fire_interval_secs: f32,
    projectile_speed: f32,
    range: f32,
    size: (f32, f32, f32),
    color: Color,
    tunables: &Tunables,
    kind: TowerKind,
) {
    let mesh = meshes.add(Cuboid::new(size.0, size.1, size.2));
    let mat = materials.add(StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });

    let _tower_entity = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_translation(Vec3::new(position.x, size.1 * 0.5, position.z)),
            Visibility::default(),
            InheritedVisibility::default(),
            Tower {
                range,
                damage,
                fire_interval_secs,
                height: size.1,
                width: size.0,
                depth: size.2,
                projectile_speed,
                last_shot: 0.0,
            },
            BuiltTower { kind },
        ))
        .id();

    // Label will be spawned by tower_damage_label_spawner system

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
pub struct TowerSpawnEffect {
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

/// Spawns damage labels for towers that don't have them yet.
pub fn tower_damage_label_spawner(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    towers: Query<(Entity, &Tower, &Transform), (With<BuiltTower>, Without<HasTowerDamageLabel>)>,
) {
    let font_handle = asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf");

    for (tower_entity, tower, _tower_transform) in towers.iter() {
        commands.spawn((
            Text::new(format!("{}", tower.damage)),
            TextFont {
                font: font_handle.clone(),
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgba(0.95, 0.95, 0.95, 0.95)),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            Visibility::default(),
            TowerDamageLabel {
                tower_entity,
                world_offset: Vec3::new(0.0, -tower.height * 0.5 + 0.5, 0.0),
            },
        ));
        // Mark tower so we don't spawn duplicate labels
        commands.entity(tower_entity).insert(HasTowerDamageLabel);
    }
}

/// Positions tower damage labels in screen space.
pub fn tower_damage_label_system(
    windows: Query<&Window>,
    cam_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    tower_query: Query<(&Transform, &Tower), With<Tower>>,
    mut labels: Query<(&TowerDamageLabel, &mut Node, &mut Visibility)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = cam_q.single() else {
        return;
    };

    let scale_factor = window.resolution.scale_factor();

    for (label, mut node, mut visibility) in labels.iter_mut() {
        // Get tower's transform and dimensions directly
        if let Ok((tower_transform, tower)) = tower_query.get(label.tower_entity) {
            let world_pos = tower_transform.translation + label.world_offset;

            // Position label in screen space
            if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, world_pos) {
                *visibility = Visibility::Visible;

                let logical_center = screen_pos / scale_factor;

                // Determine tower world half-extents from component
                let hx = tower.width * 0.5;
                let hz = tower.depth * 0.5;

                // Estimate half the on-screen width by projecting +X and +Z offsets
                let mut half_width_px: f32 = 0.0;
                if let Ok(px) =
                    camera.world_to_viewport(camera_transform, world_pos + Vec3::new(hx, 0.0, 0.0))
                {
                    half_width_px =
                        half_width_px.max(((px.x / scale_factor) - logical_center.x).abs());
                }
                if let Ok(pz) =
                    camera.world_to_viewport(camera_transform, world_pos + Vec3::new(0.0, 0.0, hz))
                {
                    half_width_px =
                        half_width_px.max(((pz.x / scale_factor) - logical_center.x).abs());
                }

                // Subtract exactly half the tower's thickness (in pixels)
                node.left = Val::Px(logical_center.x - half_width_px);
                node.top = Val::Px(logical_center.y + 10.0);
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

/// Updates tower damage label text when tower damage changes.
pub fn update_tower_damage_labels(
    towers: Query<(Entity, &Tower), Changed<Tower>>,
    mut labels: Query<(&mut Text, &TowerDamageLabel)>,
) {
    for (tower_entity, tower) in towers.iter() {
        for (mut text, label) in labels.iter_mut() {
            if label.tower_entity == tower_entity {
                text.0 = format!("{}", tower.damage);
            }
        }
    }
}

/// Cleanup labels for towers that no longer exist
pub fn cleanup_tower_damage_labels(
    mut commands: Commands,
    towers: Query<(), With<Tower>>,
    labels: Query<(Entity, &TowerDamageLabel)>,
) {
    for (entity, label) in labels.iter() {
        if towers.get(label.tower_entity).is_err() {
            commands.entity(entity).despawn();
        }
    }
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

/// Click-to-sell system. When in selling mode and left-click, sell the nearest tower
/// under the cursor within a small radius and refund 50% of its cost.
pub fn tower_selling_click(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    selling_q: Query<&SellingMode>,
    towers_q: Query<(Entity, &Transform, &BuiltTower), With<Tower>>,
    mut player_q: Query<&mut Player>,
    mut commands: Commands,
    mut building_sfx: MessageWriter<BuildingActionEvent>,
) {
    let selling_active = selling_q.iter().any(|s| s.is_active);
    if !selling_active {
        return;
    }
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, cam_tf)) = camera_q.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Some(world_point) = cursor_to_ground(camera, cam_tf, cursor_pos, 0.0) else {
        return;
    };

    // Find nearest tower within threshold on XZ plane
    let mut best: Option<(Entity, TowerKind, f32, Vec3)> = None;
    for (entity, transform, built) in towers_q.iter() {
        let tower_pos = transform.translation;
        let dx = tower_pos.x - world_point.x;
        let dz = tower_pos.z - world_point.z;
        let d2 = dx * dx + dz * dz;
        if d2 <= 4.0 {
            // threshold radius ~2.0
            if best.as_ref().map(|b| d2 < b.2).unwrap_or(true) {
                best = Some((entity, built.kind, d2, tower_pos));
            }
        }
    }

    if let Some((entity, kind, _, pos)) = best {
        // Labels are children and will be automatically despawned with the tower

        if let Ok(mut player) = player_q.single_mut() {
            let (wood_cost, rock_cost) = kind.cost();
            let wood_refund = wood_cost / 2;
            let rock_refund = rock_cost / 2;
            player.wood = player.wood.saturating_add(wood_refund);
            player.rock = player.rock.saturating_add(rock_refund);
        }
        commands.entity(entity).despawn();
        // Emit building sell SFX event
        building_sfx.write(BuildingActionEvent {
            kind: BuildingActionKind::Sell,
            position: pos,
        });
    }
}
