use super::assets::CombatVfxAssets;
use crate::components::{Enemy, Tower};
use crate::constants::Tunables;
use crate::events::EnemyKilled;
use crate::materials::{ExplosionMaterial, ImpactMaterial, ProjectileMaterial};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::time::TimerMode;
use std::time::Duration;

/// Makes towers shoot the closest enemy in range at a fixed fire rate.
pub fn tower_shooting(
    time: Res<Time>,
    mut commands: Commands,
    mut tower_query: Query<(&Transform, &mut Tower)>,
    enemy_pos: Query<(&Transform, Entity), (With<Enemy>, Without<EnemyPreExplosion>)>,
    tunables: Res<Tunables>,
    mut vfx_assets: ResMut<CombatVfxAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut projectile_materials: ResMut<Assets<ProjectileMaterial>>,
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
                spawn_projectile(
                    &mut commands,
                    &mut vfx_assets,
                    &mut meshes,
                    &mut projectile_materials,
                    tower_transform.translation,
                    enemy_pos_vec,
                    enemy_entity,
                    &tunables,
                    tower.damage,
                );
                tower.last_shot = 0.0;
            }
        }
    }
}

#[derive(Component)]
pub(crate) struct Projectile {
    target: Entity,
    speed: f32,
    damage: u32,
    last_known_target_pos: Vec3,
    material: Handle<ProjectileMaterial>,
    lifetime: Timer,
}

fn spawn_projectile(
    commands: &mut Commands,
    vfx_assets: &mut CombatVfxAssets,
    meshes: &mut Assets<Mesh>,
    projectile_materials: &mut Assets<ProjectileMaterial>,
    tower_position: Vec3,
    target_position: Vec3,
    target_entity: Entity,
    tunables: &Tunables,
    damage: u32,
) {
    let spawn_pos = Vec3::new(
        tower_position.x,
        tower_position.y + tunables.tower_height * 0.35,
        tower_position.z,
    );
    let mut direction = (target_position - spawn_pos).normalize_or_zero();
    if direction.length_squared() < f32::EPSILON {
        direction = Vec3::Y;
    }

    let mesh = vfx_assets.projectile_mesh(meshes);
    let material = projectile_materials.add(ProjectileMaterial::new(
        Color::srgba(1.0, 0.85, 0.45, 0.95),
        1.15,
    ));

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material.clone()),
        Transform {
            translation: spawn_pos,
            rotation: Quat::from_rotation_arc(Vec3::Y, direction.normalize_or_zero()),
            scale: Vec3::splat(0.55),
        },
        GlobalTransform::default(),
        Visibility::default(),
        Projectile {
            target: target_entity,
            speed: tunables.projectile_speed,
            damage,
            last_known_target_pos: target_position,
            material,
            lifetime: Timer::from_seconds(tunables.projectile_lifetime_secs, TimerMode::Once),
        },
    ));
}

pub fn projectile_system(
    time: Res<Time>,
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile, &mut Transform), Without<Enemy>>,
    enemy_pose_query: Query<&Transform, (With<Enemy>, Without<EnemyPreExplosion>)>,
    mut enemy_hit_query: Query<
        (
            &mut Enemy,
            &MeshMaterial3d<StandardMaterial>,
            Option<&mut EnemyHitFlash>,
        ),
        (With<Enemy>, Without<EnemyPreExplosion>),
    >,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut projectile_materials: ResMut<Assets<ProjectileMaterial>>,
    mut impact_materials: ResMut<Assets<ImpactMaterial>>,
    mut vfx_assets: ResMut<CombatVfxAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    tunables: Res<Tunables>,
) {
    for (entity, mut projectile, mut transform) in projectile_query.iter_mut() {
        projectile.lifetime.tick(time.delta());
        if projectile.lifetime.just_finished() {
            cleanup_projectile(
                &mut commands,
                entity,
                &projectile.material,
                &mut projectile_materials,
            );
            continue;
        }

        let (target_position, target_alive) = match enemy_pose_query.get(projectile.target) {
            Ok(tf) => {
                projectile.last_known_target_pos = tf.translation;
                (tf.translation, true)
            }
            Err(_) => (projectile.last_known_target_pos, false),
        };

        let to_target = target_position - transform.translation;
        let distance = to_target.length();
        let step = projectile.speed * time.delta_secs();

        if distance <= tunables.projectile_hit_radius || distance <= step {
            let impact_point = if target_alive {
                target_position
            } else {
                projectile.last_known_target_pos
            } + Vec3::new(0.0, 0.2, 0.0);

            if target_alive {
                handle_projectile_hit(
                    &mut commands,
                    projectile.target,
                    projectile.damage,
                    impact_point,
                    &mut enemy_hit_query,
                    &mut standard_materials,
                    &tunables,
                );
            }

            spawn_impact_flash(
                &mut commands,
                &mut vfx_assets,
                &mut meshes,
                &mut impact_materials,
                impact_point,
                &tunables,
            );

            if target_alive {
                spawn_damage_number(&mut commands, &tunables, projectile.damage, impact_point);
            }

            cleanup_projectile(
                &mut commands,
                entity,
                &projectile.material,
                &mut projectile_materials,
            );
            continue;
        }

        if distance > f32::EPSILON {
            let direction = to_target / distance;
            transform.translation += direction * step;
            transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction);
        }
    }
}

fn handle_projectile_hit(
    commands: &mut Commands,
    enemy_entity: Entity,
    damage: u32,
    impact_point: Vec3,
    enemy_hit_query: &mut Query<
        (
            &mut Enemy,
            &MeshMaterial3d<StandardMaterial>,
            Option<&mut EnemyHitFlash>,
        ),
        (With<Enemy>, Without<EnemyPreExplosion>),
    >,
    standard_materials: &mut Assets<StandardMaterial>,
    tunables: &Tunables,
) {
    if let Ok((mut enemy, material_handle, flash_opt)) = enemy_hit_query.get_mut(enemy_entity) {
        enemy.health = enemy.health.saturating_sub(damage);
        let remaining_health = enemy.health;
        drop(enemy);

        let mat_handle = material_handle.0.clone();
        let original_color = standard_materials
            .get(&mat_handle)
            .map(|mat| mat.base_color)
            .unwrap_or(Color::srgb(0.9, 0.1, 0.1));

        let lethal_hit = remaining_health == 0;

        if lethal_hit {
            if let Some(mat) = standard_materials.get_mut(&mat_handle) {
                mat.base_color = Color::srgba(0.9, 1.0, 0.9, 1.0);
            }
            commands.entity(enemy_entity).remove::<EnemyHitFlash>();
            start_enemy_pre_explosion(
                commands,
                enemy_entity,
                mat_handle,
                original_color,
                impact_point,
                standard_materials,
                tunables,
            );
        } else {
            if let Some(mut flash) = flash_opt {
                flash
                    .timer
                    .set_duration(Duration::from_secs_f32(tunables.enemy_flash_duration_secs));
                flash.timer.reset();
            } else {
                commands.entity(enemy_entity).insert(EnemyHitFlash {
                    timer: Timer::from_seconds(tunables.enemy_flash_duration_secs, TimerMode::Once),
                    original_color,
                    material: mat_handle.clone(),
                });
            }

            if let Some(mat) = standard_materials.get_mut(&mat_handle) {
                mat.base_color = Color::WHITE;
            }
        }
    }
}

fn start_enemy_pre_explosion(
    commands: &mut Commands,
    enemy_entity: Entity,
    material_handle: Handle<StandardMaterial>,
    original_color: Color,
    explosion_origin: Vec3,
    standard_materials: &mut Assets<StandardMaterial>,
    tunables: &Tunables,
) {
    if let Some(mat) = standard_materials.get_mut(&material_handle) {
        mat.base_color = Color::srgba(0.35, 0.9, 0.35, 1.0);
    }

    commands.entity(enemy_entity).insert(EnemyPreExplosion {
        timer: Timer::from_seconds(tunables.enemy_pre_explosion_duration_secs, TimerMode::Once),
        original_color,
        material: material_handle,
        flashes: tunables.enemy_pre_explosion_flashes,
        last_flash_state: true,
        explosion_origin,
    });
}

fn spawn_impact_flash(
    commands: &mut Commands,
    vfx_assets: &mut CombatVfxAssets,
    meshes: &mut Assets<Mesh>,
    impact_materials: &mut Assets<ImpactMaterial>,
    impact_point: Vec3,
    tunables: &Tunables,
) {
    let mesh = vfx_assets.impact_mesh(meshes);
    let material = impact_materials.add(ImpactMaterial::new(Color::srgba(1.0, 0.65, 0.3, 0.9)));
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material.clone()),
        Transform {
            translation: Vec3::new(impact_point.x, impact_point.y + 0.02, impact_point.z),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            scale: Vec3::splat(0.8),
        },
        GlobalTransform::default(),
        Visibility::default(),
        ImpactEffect {
            timer: Timer::from_seconds(tunables.impact_effect_duration_secs, TimerMode::Once),
            material,
        },
    ));
}

fn spawn_explosion_effect(
    commands: &mut Commands,
    vfx_assets: &mut CombatVfxAssets,
    meshes: &mut Assets<Mesh>,
    explosion_materials: &mut Assets<ExplosionMaterial>,
    origin: Vec3,
    tunables: &Tunables,
) {
    let mesh = vfx_assets.explosion_mesh(meshes);
    let material =
        explosion_materials.add(ExplosionMaterial::new(Color::srgba(1.0, 0.8, 0.45, 0.95)));
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material.clone()),
        Transform {
            translation: Vec3::new(origin.x, origin.y + 0.05, origin.z),
            scale: Vec3::splat(0.6),
            ..Default::default()
        },
        GlobalTransform::default(),
        Visibility::default(),
        ExplosionEffect {
            timer: Timer::from_seconds(tunables.explosion_effect_duration_secs, TimerMode::Once),
            material,
        },
    ));
}

fn spawn_damage_number(commands: &mut Commands, tunables: &Tunables, damage: u32, point: Vec3) {
    let world_position = point + Vec3::new(0.0, tunables.damage_number_spawn_height, 0.0);

    commands.spawn((
        DamageNumber {
            timer: Timer::from_seconds(tunables.damage_number_lifetime_secs, TimerMode::Once),
            world_position,
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
        Text::new(damage.to_string()),
        TextFont {
            font_size: tunables.damage_number_font_size,
            ..default()
        },
        TextColor(Color::srgba(1.0, 0.9, 0.55, 1.0)),
        Visibility::Hidden,
    ));
}

fn cleanup_projectile(
    commands: &mut Commands,
    entity: Entity,
    material: &Handle<ProjectileMaterial>,
    projectile_materials: &mut Assets<ProjectileMaterial>,
) {
    projectile_materials.remove(material.id());
    commands.entity(entity).despawn();
}

#[derive(Component)]
pub(crate) struct ImpactEffect {
    timer: Timer,
    material: Handle<ImpactMaterial>,
}

#[derive(Component)]
pub(crate) struct ExplosionEffect {
    timer: Timer,
    material: Handle<ExplosionMaterial>,
}

#[derive(Component)]
pub(crate) struct DamageNumber {
    timer: Timer,
    world_position: Vec3,
}

#[derive(Component)]
pub(crate) struct EnemyHitFlash {
    timer: Timer,
    original_color: Color,
    material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub(crate) struct EnemyPreExplosion {
    timer: Timer,
    original_color: Color,
    material: Handle<StandardMaterial>,
    flashes: f32,
    last_flash_state: bool,
    explosion_origin: Vec3,
}

pub fn impact_effect_system(
    time: Res<Time>,
    mut commands: Commands,
    mut effects: Query<(Entity, &mut ImpactEffect, &mut Transform)>,
    mut impact_materials: ResMut<Assets<ImpactMaterial>>,
) {
    for (entity, mut effect, mut transform) in effects.iter_mut() {
        effect.timer.tick(time.delta());
        let duration = effect.timer.duration().as_secs_f32().max(f32::EPSILON);
        let progress = (effect.timer.elapsed().as_secs_f32() / duration).clamp(0.0, 1.0);
        transform.scale = Vec3::splat(0.8 + progress * 1.6);

        if let Some(mat) = impact_materials.get_mut(&effect.material) {
            mat.data.progress = progress;
        }

        if effect.timer.just_finished() {
            impact_materials.remove(effect.material.id());
            commands.entity(entity).despawn();
        }
    }
}

pub fn explosion_effect_system(
    time: Res<Time>,
    mut commands: Commands,
    mut effects: Query<(Entity, &mut ExplosionEffect, &mut Transform)>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
    tunables: Res<Tunables>,
) {
    let base_scale = 0.6;
    for (entity, mut effect, mut transform) in effects.iter_mut() {
        effect.timer.tick(time.delta());
        let duration = effect.timer.duration().as_secs_f32().max(f32::EPSILON);
        let progress = (effect.timer.elapsed().as_secs_f32() / duration).clamp(0.0, 1.0);
        let eased = progress.powf(0.7);
        let target_scale =
            base_scale + eased * (tunables.explosion_effect_max_scale - base_scale).max(0.0);
        transform.scale = Vec3::splat(target_scale);

        if let Some(mat) = explosion_materials.get_mut(&effect.material) {
            mat.data.progress = progress;
            mat.data.glow = 1.4 - progress * 0.8;
        }

        if effect.timer.just_finished() {
            explosion_materials.remove(effect.material.id());
            commands.entity(entity).despawn();
        }
    }
}

pub fn damage_number_system(
    time: Res<Time>,
    mut commands: Commands,
    windows: Query<&Window>,
    cam_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut numbers: Query<(
        Entity,
        &mut DamageNumber,
        &mut Node,
        &mut TextColor,
        &mut Visibility,
    )>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = cam_q.single() else {
        return;
    };

    let scale_factor = window.resolution.scale_factor();
    let logical_height = window.resolution.height();

    for (entity, mut number, mut node, mut color, mut visibility) in numbers.iter_mut() {
        number.timer.tick(time.delta());

        if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, number.world_position) {
            *visibility = Visibility::Visible;
            let logical_pos = screen_pos / scale_factor;
            let half = 12.0;
            node.left = Val::Px(logical_pos.x - half);
            node.top = Val::Px(logical_height - logical_pos.y - half);
        } else {
            *visibility = Visibility::Hidden;
        }

        let duration = number.timer.duration().as_secs_f32().max(f32::EPSILON);
        let ratio = (number.timer.elapsed().as_secs_f32() / duration).clamp(0.0, 1.0);
        let alpha = (1.0 - ratio).powf(1.4);
        color.0 = color.0.with_alpha(alpha);

        if number.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn enemy_pre_explosion_system(
    time: Res<Time>,
    mut commands: Commands,
    mut pre_explosions: Query<(Entity, &mut EnemyPreExplosion)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut vfx_assets: ResMut<CombatVfxAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
    children_query: Query<&Children>,
    mut enemy_killed_events: MessageWriter<EnemyKilled>,
    tunables: Res<Tunables>,
) {
    for (entity, mut pre) in pre_explosions.iter_mut() {
        pre.timer.tick(time.delta());
        let duration = pre.timer.duration().as_secs_f32().max(f32::EPSILON);
        let progress = (pre.timer.elapsed().as_secs_f32() / duration).clamp(0.0, 1.0);
        let flash_phase = (progress * pre.flashes).fract();
        let flash_on = flash_phase < 0.5;

        if flash_on != pre.last_flash_state {
            if let Some(mat) = materials.get_mut(&pre.material) {
                if flash_on {
                    mat.base_color = Color::WHITE;
                } else {
                    mat.base_color = Color::srgba(0.35, 0.9, 0.35, 1.0);
                }
            }
            pre.last_flash_state = flash_on;
        }

        if pre.timer.just_finished() {
            if let Some(mat) = materials.get_mut(&pre.material) {
                mat.base_color = pre.original_color;
            }

            spawn_explosion_effect(
                &mut commands,
                &mut vfx_assets,
                &mut meshes,
                &mut explosion_materials,
                pre.explosion_origin,
                &tunables,
            );

            enemy_killed_events.write(EnemyKilled {
                position: pre.explosion_origin,
            });

            despawn_entity_recursive(&mut commands, entity, &children_query);
        }
    }
}

fn despawn_entity_recursive(
    commands: &mut Commands,
    root: Entity,
    children_query: &Query<&Children>,
) {
    let mut stack = Vec::new();
    stack.push(root);

    while let Some(entity) = stack.pop() {
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                stack.push(child);
            }
        }
        commands.entity(entity).despawn();
    }
}

pub fn enemy_flash_system(
    time: Res<Time>,
    mut commands: Commands,
    mut flashes: Query<(Entity, &mut EnemyHitFlash)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut flash) in flashes.iter_mut() {
        flash.timer.tick(time.delta());
        let duration = flash.timer.duration().as_secs_f32().max(f32::EPSILON);
        let ratio = (flash.timer.elapsed().as_secs_f32() / duration).clamp(0.0, 1.0);
        let whiteness = (1.0 - ratio).powf(0.4);

        if let Some(mat) = materials.get_mut(&flash.material) {
            let srgb = flash.original_color.to_srgba();
            let blended = Color::srgba(
                srgb.red + (1.0 - srgb.red) * whiteness,
                srgb.green + (1.0 - srgb.green) * whiteness,
                srgb.blue + (1.0 - srgb.blue) * whiteness,
                srgb.alpha,
            );
            mat.base_color = blended;
        }

        if flash.timer.just_finished() {
            if let Some(mat) = materials.get_mut(&flash.material) {
                mat.base_color = flash.original_color;
            }
            commands.entity(entity).remove::<EnemyHitFlash>();
        }
    }
}


