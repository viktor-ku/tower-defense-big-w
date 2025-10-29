use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;

use super::definitions::{BuildCatalog, BuildDefinitionId};
use crate::components::Player;

#[derive(Resource, Clone)]
pub struct GridConfig {
    pub cell: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self { cell: 1.0 }
    }
}

#[derive(Resource, Default, Clone)]
pub struct PlacementState {
    pub active: Option<BuildDefinitionId>,
    pub rotation_degrees: f32,
    pub ghost: Option<Entity>,
}

#[derive(Component)]
pub struct BuildGhost;

pub fn placement_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<PlacementState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ghost_tf_q: Query<&Transform, With<BuildGhost>>,
    mut player_q: Query<&mut Player, With<Player>>,
    catalog: Res<BuildCatalog>,
) {
    if state.active.is_none() {
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right) {
        // Cancel
        if let Some(ghost) = state.ghost.take() {
            if commands.get_entity(ghost).is_ok() {
                commands.entity(ghost).despawn();
            }
        }
        state.active = None;
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        state.rotation_degrees = (state.rotation_degrees + 90.0) % 360.0;
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        state.rotation_degrees = (state.rotation_degrees - 90.0) % 360.0;
    }

    if mouse.just_pressed(MouseButton::Left) {
        // For now, confirm spawns a small colored cube at the ghost's position
        if let Some(ghost) = state.ghost {
            if let Ok(tf) = ghost_tf_q.get(ghost) {
                // Affordability: treat cost as wood
                if let Some(def) = state
                    .active
                    .and_then(|id| catalog.items.iter().find(|d| d.id == id))
                {
                    if let Ok(mut player) = player_q.single_mut() {
                        if player.wood < def.cost {
                            return;
                        }
                        player.wood -= def.cost;
                    }
                }
                let translation = tf.translation;
                let mesh = meshes.add(bevy::math::primitives::Cuboid::new(1.0, 0.2, 1.0));
                let mat = materials.add(StandardMaterial {
                    base_color: Color::srgba(0.12, 0.47, 0.95, 0.8),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_translation(translation),
                ));
            }
        }
    }
}

pub fn update_placement(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut state: ResMut<PlacementState>,
    catalog: Res<BuildCatalog>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid: Res<GridConfig>,
    mut ghost_tf_q: Query<&mut Transform, With<BuildGhost>>,
) {
    let Some(_active_id) = state.active else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, cam_tf)) = camera_q.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Some(mut world) = cursor_to_ground(camera, cam_tf, cursor_pos, 0.0) else {
        return;
    };

    // Snap to grid
    let s = grid.cell.max(0.1);
    world.x = (world.x / s).round() * s;
    world.z = (world.z / s).round() * s;

    // Spawn ghost if needed
    if state.ghost.is_none() {
        let mesh = meshes.add(bevy::math::primitives::Cuboid::new(s, 0.1, s));
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.12, 0.66, 0.32, 0.5),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let e = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_translation(world),
                BuildGhost,
            ))
            .id();
        state.ghost = Some(e);
    } else if let Some(ghost) = state.ghost {
        if let Ok(mut tf) = ghost_tf_q.get_mut(ghost) {
            *tf = Transform::from_translation(world);
        }
    }
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

pub fn cleanup_placement(mut commands: Commands, mut state: ResMut<PlacementState>) {
    if let Some(ghost) = state.ghost.take() {
        if commands.get_entity(ghost).is_ok() {
            commands.entity(ghost).despawn();
        }
    }
    state.active = None;
}
