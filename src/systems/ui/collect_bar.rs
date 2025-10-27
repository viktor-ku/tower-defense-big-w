use crate::components::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct CollectUiRoot;

#[derive(Component)]
pub struct CollectUiFill;

#[derive(Resource, Default)]
pub struct CollectUiState {
    pub bar_entity: Option<Entity>,
    pub target: Option<Entity>,
}

#[allow(clippy::too_many_arguments)]
pub fn manage_collect_bar_ui(
    mut commands: Commands,
    mut state: ResMut<CollectUiState>,
    progress: Res<CurrentCollectProgress>,
    cam_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    target_tf_q: Query<&GlobalTransform>,
    windows: Query<&Window>,
    mut root_q: Query<&mut Node, With<CollectUiRoot>>,
    mut fill_q: Query<&mut Node, (With<CollectUiFill>, Without<CollectUiRoot>)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, cam_tf)) = cam_q.single() else {
        return;
    };

    if progress.target != state.target {
        if let Some(e) = state.bar_entity.take()
            && let Ok(mut ec) = commands.get_entity(e)
        {
            ec.despawn();
        }
        state.target = progress.target;

        if progress.target.is_some() {
            let entity = commands
                .spawn((
                    CollectUiRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Px(120.0),
                        height: Val::Px(10.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        CollectUiFill,
                        Node {
                            width: Val::Px(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.85, 0.2)),
                    ));
                })
                .id();
            state.bar_entity = Some(entity);
        }
    }

    if let (Some(target), Some(root_e)) = (progress.target, state.bar_entity)
        && let Ok(target_tf) = target_tf_q.get(target)
    {
        let world_pos = target_tf.translation() + Vec3::Y * 2.5;
        if let Ok(mut screen) = camera.world_to_viewport(cam_tf, world_pos) {
            screen.y = window.height() - screen.y;
            if let Ok(mut node) = root_q.get_mut(root_e) {
                node.left = Val::Px(screen.x - 60.0);
                node.top = Val::Px(screen.y - 20.0);
            }
            if let Ok(mut fill) = fill_q.single_mut() {
                let px = (progress.progress.clamp(0.0, 1.0)) * 120.0;
                fill.width = Val::Px(px);
            }
        }
    }
}
