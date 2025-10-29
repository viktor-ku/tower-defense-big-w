use super::assets::CombatVfxAssets;
use crate::audio::{TowerShotEvent, TowerShotKind};
use crate::components::{BuiltTower, Enemy, EnemyKind, Player, Tower, TowerKind};
use crate::constants::Tunables;
use crate::events::{DamageDealt, EnemyKilled};
use crate::materials::ImpactMaterial;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::time::TimerMode;
use std::time::Duration;

/// Makes towers shoot the closest enemy in range at a fixed fire rate.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn tower_shooting(
    time: Res<Time>,
    mut commands: Commands,
    mut tower_query: Query<(&Transform, &mut Tower, Option<&BuiltTower>)>,
    enemy_pos: Query<(&Transform, Entity), (With<Enemy>, Without<EnemyFadeOut>)>,
    tunables: Res<Tunables>,
    vfx_assets: Res<CombatVfxAssets>,
    mut shot_events: MessageWriter<TowerShotEvent>,
) {
    for (tower_transform, mut tower, built_kind_opt) in tower_query.iter_mut() {
        tower.last_shot += time.delta_secs();

        if tower.last_shot >= tower.fire_interval_secs {
            // Find closest enemy within range without per-frame allocations
            let origin = tower_transform.translation;
            let mut best_entity: Option<(Vec3, Entity)> = None;
            let mut best_dist: f32 = tower.range;
            for (enemy_transform, entity) in enemy_pos.iter() {
                let pos = enemy_transform.translation;
                let d = origin.distance(pos);
                if d <= best_dist {
                    best_dist = d;
                    best_entity = Some((pos, entity));
                }
            }

            if let Some((enemy_pos_vec, enemy_entity)) = best_entity {
                spawn_projectile(
                    &mut commands,
                    &vfx_assets,
                    tower_transform.translation,
                    enemy_pos_vec,
                    enemy_entity,
                    &tunables,
                    tower.damage,
                    tower.height,
                    tower.projectile_speed,
                );
                // Emit tower shot audio event from tower position
                let kind = match built_kind_opt.map(|b| b.kind).unwrap_or(TowerKind::Bow) {
                    TowerKind::Bow => TowerShotKind::Bow,
                    TowerKind::Crossbow => TowerShotKind::Crossbow,
                };
                shot_events.write(TowerShotEvent {
                    kind,
                    position: tower_transform.translation,
                });
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
    lifetime: Timer,
}

#[allow(clippy::too_many_arguments)]
fn spawn_projectile(
    commands: &mut Commands,
    vfx_assets: &CombatVfxAssets,
    tower_position: Vec3,
    target_position: Vec3,
    target_entity: Entity,
    tunables: &Tunables,
    damage: u32,
    tower_height: f32,
    projectile_speed: f32,
) {
    let spawn_pos = Vec3::new(
        tower_position.x,
        tower_position.y + tower_height * 0.35,
        tower_position.z,
    );
    let mut direction = (target_position - spawn_pos).normalize_or_zero();
    if direction.length_squared() < f32::EPSILON {
        direction = Vec3::Y;
    }

    let mesh = vfx_assets
        .projectile_mesh_handle()
        .expect("CombatVfxAssets not initialized: projectile_mesh");

    commands.spawn((
        Mesh3d(mesh),
        // Use a solid unlit white StandardMaterial for the main projectile visibility
        MeshMaterial3d(
            vfx_assets
                .projectile_white_material_handle()
                .expect("CombatVfxAssets not initialized: projectile_white_material"),
        ),
        Transform {
            translation: spawn_pos,
            rotation: Quat::from_rotation_arc(Vec3::Y, direction.normalize_or_zero()),
            // Further elongated to resemble an arrow/bolt (Y is forward axis)
            scale: Vec3::new(0.12, 2.4, 0.12),
        },
        GlobalTransform::default(),
        Visibility::default(),
        Projectile {
            target: target_entity,
            speed: projectile_speed,
            damage,
            last_known_target_pos: target_position,
            lifetime: Timer::from_seconds(tunables.projectile_lifetime_secs, TimerMode::Once),
        },
    ));
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn projectile_system(
    time: Res<Time>,
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile, &mut Transform), Without<Enemy>>,
    enemy_pose_query: Query<&GlobalTransform, (With<Enemy>, Without<EnemyFadeOut>)>,
    mut enemy_hit_query: Query<
        (
            &mut Enemy,
            &MeshMaterial3d<StandardMaterial>,
            Option<&mut EnemyHitFlash>,
        ),
        With<Enemy>,
    >,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut impact_materials: ResMut<Assets<ImpactMaterial>>,
    vfx_assets: Res<CombatVfxAssets>,
    tunables: Res<Tunables>,
    mut damage_dealt_events: MessageWriter<DamageDealt>,
) {
    for (entity, mut projectile, mut transform) in projectile_query.iter_mut() {
        projectile.lifetime.tick(time.delta());
        if projectile.lifetime.just_finished() {
            cleanup_projectile(&mut commands, entity);
            continue;
        }

        let (target_position, target_alive) = match enemy_pose_query.get(projectile.target) {
            Ok(tf) => {
                let world_pos = tf.translation();
                projectile.last_known_target_pos = world_pos;
                (world_pos, true)
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
                damage_dealt_events.write(DamageDealt {
                    enemy: projectile.target,
                    amount: projectile.damage,
                });
            }

            spawn_impact_flash(
                &mut commands,
                &vfx_assets,
                &mut impact_materials,
                impact_point,
                &tunables,
            );

            // Old damage number spawn removed; now handled via DamageDealt events

            cleanup_projectile(&mut commands, entity);
            continue;
        }

        if distance > f32::EPSILON {
            let direction = to_target / distance;
            transform.translation += direction * step;
            transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction);
        }

        // no trailing
    }
}

// trailing removed

// trailing removed

#[allow(clippy::type_complexity)]
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
        With<Enemy>,
    >,
    standard_materials: &mut Assets<StandardMaterial>,
    tunables: &Tunables,
) {
    if let Ok((mut enemy, material_handle, flash_opt)) = enemy_hit_query.get_mut(enemy_entity) {
        enemy.health = enemy.health.saturating_sub(damage);
        let remaining_health = enemy.health;

        let mat_handle = material_handle.0.clone();
        let original_color = standard_materials
            .get(&mat_handle)
            .map(|mat| mat.base_color)
            .unwrap_or(Color::srgb(0.9, 0.1, 0.1));

        let lethal_hit = remaining_health == 0;

        if lethal_hit {
            // Stop any hit flash and start a fade-out instead of pre-explosion blink
            commands.entity(enemy_entity).remove::<EnemyHitFlash>();

            if let Some(mat) = standard_materials.get_mut(&mat_handle) {
                mat.alpha_mode = AlphaMode::Blend;
                let base = original_color.to_srgba();
                mat.base_color = Color::srgba(base.red, base.green, base.blue, 1.0);
            }

            commands.entity(enemy_entity).insert(EnemyFadeOut {
                timer: Timer::from_seconds(tunables.enemy_fade_out_duration_secs, TimerMode::Once),
                material: mat_handle,
                original_color,
                death_position: impact_point,
            });
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

// Pre-explosion blink removed; replaced with fade-out

fn spawn_impact_flash(
    commands: &mut Commands,
    vfx_assets: &CombatVfxAssets,
    impact_materials: &mut Assets<ImpactMaterial>,
    impact_point: Vec3,
    tunables: &Tunables,
) {
    let mesh = vfx_assets
        .impact_mesh_handle()
        .expect("CombatVfxAssets not initialized: impact_mesh");
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

// Explosion effect spawner removed; we no longer spawn explosion VFX on enemy death

pub fn damage_dealt_spawn_text_system(
    mut commands: Commands,
    tunables: Res<Tunables>,
    mut events: MessageReader<DamageDealt>,
    enemy_pose_query: Query<&GlobalTransform, With<Enemy>>,
    asset_server: Res<AssetServer>,
) {
    for evt in events.read() {
        if let Ok(tf) = enemy_pose_query.get(evt.enemy) {
            let pos = tf.translation() + Vec3::new(0.0, tunables.damage_number_spawn_height, 0.0);
            // Choose a small random UI offset to prevent overlap (left/right/top/bottom)
            let dir = rand::random::<u8>() % 4;
            let offset_px = match dir {
                0 => Vec2::new(10.0, 0.0),  // right
                1 => Vec2::new(-10.0, 0.0), // left
                2 => Vec2::new(0.0, 10.0),  // down
                _ => Vec2::new(0.0, -10.0), // up
            };
            commands.spawn((
                DamageNumber {
                    timer: Timer::from_seconds(
                        tunables.damage_number_lifetime_secs,
                        TimerMode::Once,
                    ),
                    world_position: pos,
                    ui_offset: offset_px,
                },
                Text::new(evt.amount.to_string()),
                TextFont {
                    font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                    font_size: tunables.damage_number_font_size,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.6, 0.9)),
            ));
        }
    }
}

fn cleanup_projectile(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).despawn();
}

#[derive(Component)]
pub(crate) struct ImpactEffect {
    timer: Timer,
    material: Handle<ImpactMaterial>,
}

// trailing removed

#[derive(Component)]
pub(crate) struct DamageNumber {
    timer: Timer,
    world_position: Vec3,
    ui_offset: Vec2,
}

#[derive(Component)]
pub(crate) struct EnemyHitFlash {
    timer: Timer,
    original_color: Color,
    material: Handle<StandardMaterial>,
}

// EnemyPreExplosion removed; replaced by EnemyFadeOut

#[derive(Component)]
pub(crate) struct EnemyFadeOut {
    timer: Timer,
    material: Handle<StandardMaterial>,
    original_color: Color,
    death_position: Vec3,
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

// explosion effect system removed

// trailing removed

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

    for (entity, mut number, mut node, mut color, mut visibility) in numbers.iter_mut() {
        number.timer.tick(time.delta());

        if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, number.world_position) {
            *visibility = Visibility::Visible;

            let margin = 10.0;

            // Convert to logical UI coordinates: top-left origin
            let logical_pos = screen_pos / scale_factor;
            node.left = Val::Px(logical_pos.x - margin + number.ui_offset.x);
            node.top = Val::Px(logical_pos.y - margin + number.ui_offset.y);
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

#[allow(clippy::too_many_arguments)]
pub fn enemy_fade_out_system(
    time: Res<Time>,
    mut commands: Commands,
    mut fading: Query<(Entity, &mut EnemyFadeOut)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    children_query: Query<&Children>,
    mut enemy_killed_events: MessageWriter<EnemyKilled>,
    enemy_kind_q: Query<&EnemyKind>,
    mut player_q: Query<&mut Player>,
    asset_server: Res<AssetServer>,
    tunables: Res<Tunables>,
) {
    for (entity, mut fade) in fading.iter_mut() {
        fade.timer.tick(time.delta());
        let duration = fade.timer.duration().as_secs_f32().max(f32::EPSILON);
        let progress = (fade.timer.elapsed().as_secs_f32() / duration).clamp(0.0, 1.0);

        if let Some(mat) = materials.get_mut(&fade.material) {
            let c = fade.original_color.to_srgba();
            let alpha = 1.0 - progress;
            mat.base_color = Color::srgba(c.red, c.green, c.blue, alpha);
            mat.alpha_mode = AlphaMode::Blend;
        }

        if fade.timer.just_finished() {
            // Credit currency based on enemy kind
            let silver_award: u64 = match enemy_kind_q.get(entity).ok().copied() {
                Some(EnemyKind::Minion) => 1u64,
                Some(EnemyKind::Zombie) => 2u64,
                Some(EnemyKind::Boss) => 5u64,
                None => 1u64,
            };

            let gold_award: u64 = if rand::random::<f32>() < 0.05 {
                1u64
            } else {
                0u64
            };

            if let Ok(mut player) = player_q.single_mut() {
                player.silver = player.silver.saturating_add(silver_award);
                if gold_award > 0 {
                    player.gold = player.gold.saturating_add(1u64);
                }
            }

            // Spawn floating reward texts
            let pos =
                fade.death_position + Vec3::new(0.0, tunables.damage_number_spawn_height, 0.0);
            let dir = rand::random::<u8>() % 4;
            let offset_px = match dir {
                0 => Vec2::new(10.0, 0.0),
                1 => Vec2::new(-10.0, 0.0),
                2 => Vec2::new(0.0, 10.0),
                _ => Vec2::new(0.0, -10.0),
            };

            commands.spawn((
                DamageNumber {
                    timer: Timer::from_seconds(
                        tunables.damage_number_lifetime_secs,
                        TimerMode::Once,
                    ),
                    world_position: pos,
                    ui_offset: offset_px,
                },
                Text::new(format!("+{}S", silver_award)),
                TextFont {
                    font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                    font_size: tunables.damage_number_font_size,
                    ..default()
                },
                TextColor(Color::srgba(0.80, 0.82, 0.90, 0.95)),
            ));

            if gold_award > 0 {
                let dir2 = (dir + 1) % 4; // different offset direction
                let offset_px2 = match dir2 {
                    0 => Vec2::new(10.0, 0.0),
                    1 => Vec2::new(-10.0, 0.0),
                    2 => Vec2::new(0.0, 10.0),
                    _ => Vec2::new(0.0, -10.0),
                };
                commands.spawn((
                    DamageNumber {
                        timer: Timer::from_seconds(
                            tunables.damage_number_lifetime_secs,
                            TimerMode::Once,
                        ),
                        world_position: pos,
                        ui_offset: offset_px2,
                    },
                    Text::new("+1G".to_string()),
                    TextFont {
                        font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                        font_size: tunables.damage_number_font_size,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 0.92, 0.35, 0.98)),
                ));
            }

            enemy_killed_events.write(EnemyKilled {
                position: fade.death_position,
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
