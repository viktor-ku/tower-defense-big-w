use crate::components::*;
use crate::events::*;
use bevy::prelude::*;

// Observer-based logging for gameplay events (Bevy 0.17)
pub fn on_resource_collected(trigger: On<ResourceCollected>) {
    let e = trigger.event();
    info!("Resource collected: {:?} x{}", e.kind, e.amount);
}

pub fn on_tower_built(trigger: On<TowerBuilt>) {
    let e = trigger.event();
    info!("Tower built at: {:?}", e.position);
}

pub fn on_enemy_spawned(trigger: On<EnemySpawned>) {
    let e = trigger.event();
    info!("Enemy spawned at: {:?}", e.position);
}

pub fn on_enemy_killed(trigger: On<EnemyKilled>) {
    let e = trigger.event();
    info!("Enemy killed at: {:?}", e.position);
}

/// Updates the persistent HUD health bar for the village.
pub fn village_health_hud(
    windows: Query<&Window>,
    village_query: Query<&Village>,
    mut fill_query: Query<&mut Node, With<HealthBar>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    if let Ok(village) = village_query.single() {
        let health_percentage = village.health as f32 / village.max_health as f32;
        let total_width_px = window.width() * 0.6; // 60% of screen
        let fill_width_px = total_width_px * health_percentage.clamp(0.0, 1.0);

        for mut node in fill_query.iter_mut() {
            // The green bar should always start from the left and shrink from the right
            // So we only change the width, keeping left position fixed
            node.width = Val::Px(fill_width_px);
        }
    }
}

/// Marker component for the health bar fill node.
#[derive(Component)]
pub struct HealthBar;

/// Spawns the persistent on-screen HUD with a background and a fill node.
pub fn spawn_village_health_bar(mut commands: Commands) {
    // Root container: centered horizontally using left: 20% and width: 60%
    commands
        .spawn((
            Node {
                left: Val::Percent(20.0),
                top: Val::Px(20.0),
                width: Val::Percent(60.0),
                height: Val::Px(40.0),
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                ..default()
            },
            // White outline
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.85)),
            BorderColor::all(Color::srgba(0.95, 0.95, 0.98, 0.55)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.22, 0.75, 0.28, 0.95)),
                HealthBar,
            ));
        });
}

/// Marker for the wood counter text node.
#[derive(Component)]
pub struct WoodCounterText;

/// Marker for the rock counter text node.
#[derive(Component)]
pub struct RockCounterText;

/// Marker for the wave counter text node.
#[derive(Component)]
pub struct WaveCounterText;

/// Marker for the wave timer text node.
#[derive(Component)]
pub struct WaveTimerText;

/// Tracks last rendered value for resource counters to avoid repeated allocations.
#[derive(Component)]
pub(crate) struct ResourceCounter {
    kind: HarvestableKind,
    last_value: u32,
}

/// Tracks last rendered wave number.
#[derive(Component)]
pub(crate) struct WaveCounterDisplay {
    last_value: u32,
}

/// Tracks last rendered timer seconds (or `None` for "Wave in progress").
#[derive(Component)]
pub(crate) struct WaveTimerDisplay {
    last_seconds: Option<u32>,
}

/// Spawns resource counters (wood and rock) at the top-left of the screen.
pub fn spawn_resource_counters(mut commands: Commands) {
    commands
        .spawn((
            Node {
                left: Val::Px(20.0),
                top: Val::Px(70.0),
                width: Val::Auto,
                height: Val::Auto,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(6.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.06, 0.92)),
            BorderColor::all(Color::srgba(0.6, 0.72, 0.9, 0.45)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Wood: 0"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::srgba(0.93, 0.86, 0.68, 1.0)),
                WoodCounterText,
                ResourceCounter {
                    kind: HarvestableKind::Wood,
                    last_value: 0,
                },
            ));

            parent.spawn((
                Text::new("Rock: 0"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::srgba(0.86, 0.88, 0.95, 1.0)),
                RockCounterText,
                ResourceCounter {
                    kind: HarvestableKind::Rock,
                    last_value: 0,
                },
            ));
        });
}

/// Spawns HUD elements for the current wave and intermission timer.
pub fn spawn_wave_hud(mut commands: Commands, wave_state: Res<WaveState>) {
    let wave_number = wave_state.upcoming_wave_number();
    let (timer_label, timer_state) = match wave_state.phase {
        WavePhase::Intermission => {
            let seconds = wave_state.remaining_intermission_secs().ceil().max(0.0) as u32;
            (
                format!("Next wave in: {}s", seconds),
                WaveTimerDisplay {
                    last_seconds: Some(seconds),
                },
            )
        }
        WavePhase::Spawning => (
            "Wave in progress".to_string(),
            WaveTimerDisplay { last_seconds: None },
        ),
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                width: Val::Auto,
                height: Val::Auto,
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(8.0),
                align_items: AlignItems::FlexEnd,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.9)),
            BorderColor::all(Color::srgba(0.75, 0.6, 0.9, 0.45)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("Wave: {}", wave_number)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgba(0.92, 0.88, 1.0, 1.0)),
                WaveCounterText,
                WaveCounterDisplay {
                    last_value: wave_number,
                },
            ));

            parent.spawn((
                Text::new(timer_label),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgba(0.78, 0.86, 0.95, 1.0)),
                WaveTimerText,
                timer_state,
            ));
        });
}

/// Updates the on-screen resource counters from the Player inventory.
pub fn update_resource_counters(
    player_q: Query<&Player>,
    mut counters: Query<(&mut Text, &mut ResourceCounter)>,
) {
    if let Ok(player) = player_q.single() {
        for (mut text, mut counter) in counters.iter_mut() {
            let value = match counter.kind {
                HarvestableKind::Wood => player.wood,
                HarvestableKind::Rock => player.rock,
            };

            if counter.last_value != value {
                counter.last_value = value;
                let label = match counter.kind {
                    HarvestableKind::Wood => "Wood",
                    HarvestableKind::Rock => "Rock",
                };
                *text = Text::new(format!("{}: {}", label, value));
            }
        }
    }
}

/// Updates the wave counter and intermission timer text.
#[allow(clippy::type_complexity)]
pub fn update_wave_hud(
    wave_state: Res<WaveState>,
    mut wave_text_q: Query<(&mut Text, &mut WaveCounterDisplay), With<WaveCounterText>>,
    mut timer_text_q: Query<
        (&mut Text, &mut WaveTimerDisplay),
        (With<WaveTimerText>, Without<WaveCounterText>),
    >,
) {
    if let Ok((mut wave_text, mut display)) = wave_text_q.single_mut() {
        let upcoming = wave_state.upcoming_wave_number();
        if display.last_value != upcoming {
            display.last_value = upcoming;
            *wave_text = Text::new(format!("Wave: {}", upcoming));
        }
    }

    if let Ok((mut timer_text, mut display)) = timer_text_q.single_mut() {
        match wave_state.phase {
            WavePhase::Intermission => {
                let seconds = wave_state.remaining_intermission_secs().ceil().max(0.0) as u32;
                if display.last_seconds != Some(seconds) {
                    display.last_seconds = Some(seconds);
                    *timer_text = Text::new(format!("Next wave in: {}s", seconds));
                }
            }
            WavePhase::Spawning => {
                if display.last_seconds.is_some() {
                    display.last_seconds = None;
                    *timer_text = Text::new("Wave in progress");
                }
            }
        }
    }
}

/// Root node for the screen-space hold progress bar.
#[derive(Component)]
pub struct CollectUiRoot;

/// Fill node for the screen-space hold progress bar.
#[derive(Component)]
pub struct CollectUiFill;

/// Tracks the UI bar entity and current target.
#[derive(Resource, Default)]
pub struct CollectUiState {
    pub bar_entity: Option<Entity>,
    pub target: Option<Entity>,
}

/// Creates/positions a screen-space progress bar above the active target.
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

    // If target changed, despawn old UI and spawn new one
    if progress.target != state.target {
        if let Some(e) = state.bar_entity.take() && let Ok(mut ec) = commands.get_entity(e) {
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

    // Update position and fill
    if let (Some(target), Some(root_e)) = (progress.target, state.bar_entity)
        && let Ok(target_tf) = target_tf_q.get(target)
    {
        let world_pos = target_tf.translation() + Vec3::Y * 2.5;
        if let Ok(mut screen) = camera.world_to_viewport(cam_tf, world_pos) {
                // Center the bar above the target
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

// =====================
// Tower selection drawer
// =====================

#[derive(Component)]
pub struct TowerDrawerRoot;

#[derive(Component)]
pub struct TowerChoiceButton {
    pub kind: TowerKind,
}

/// Spawns a right-side drawer prompting the player to choose a tower when in build mode
/// and no selection is currently chosen. Despawns it otherwise.
pub fn manage_tower_selection_drawer(
    mut commands: Commands,
    building_mode_q: Query<&BuildingMode>,
    mut selection: ResMut<TowerBuildSelection>,
    children_q: Query<&Children>,
    drawer_root_alive: Query<(), With<TowerDrawerRoot>>,
) {
    let building = building_mode_q.iter().any(|b| b.is_active);

    let need_drawer = building && selection.choice.is_none();
    let has_drawer = selection.drawer_root.is_some();

    if need_drawer && !has_drawer {
        // Spawn drawer root on the right side
        let root = commands
            .spawn((
                TowerDrawerRoot,
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    top: Val::Px(120.0),
                    width: Val::Px(360.0),
                    height: Val::Auto,
                    padding: UiRect::all(Val::Px(14.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    row_gap: Val::Px(10.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.07, 0.10, 0.96)),
                BorderColor::all(Color::srgba(0.75, 0.75, 0.85, 0.45)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Choose a tower"),
                    TextFont {
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.92, 0.92, 0.98, 1.0)),
                ));

                // Bow button
                parent
                    .spawn((
                        Button,
                        TowerChoiceButton {
                            kind: TowerKind::Bow,
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Auto,
                            padding: UiRect::all(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.14, 0.16, 0.22, 0.9)),
                        BorderColor::all(Color::srgba(0.65, 0.70, 0.85, 0.35)),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            Text::new("Bow tower\nFires quickly but does little damage"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.9, 0.92, 0.98, 1.0)),
                        ));
                    });

                // Crossbow button
                parent
                    .spawn((
                        Button,
                        TowerChoiceButton {
                            kind: TowerKind::Crossbow,
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Auto,
                            padding: UiRect::all(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.14, 0.16, 0.22, 0.9)),
                        BorderColor::all(Color::srgba(0.65, 0.70, 0.85, 0.35)),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            Text::new("Crossbow tower\nFires slowly but does lots of damage"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.9, 0.92, 0.98, 1.0)),
                        ));
                    });
            })
            .id();
        selection.drawer_root = Some(root);
    } else if !need_drawer && has_drawer && let Some(root) = selection.drawer_root.take() {
        // Only despawn if the root is still alive
        if drawer_root_alive.get(root).is_ok() {
            despawn_entity_recursive(&mut commands, root, &children_q);
        }
    }
}

/// Handles button presses in the tower selection drawer.
#[allow(clippy::type_complexity)]
pub fn handle_tower_selection_buttons(
    mut commands: Commands,
    mut selection: ResMut<TowerBuildSelection>,
    mut interactions: Query<
        (&Interaction, &TowerChoiceButton),
        (Changed<Interaction>, With<Button>),
    >,
    children_q: Query<&Children>,
) {
    for (interaction, button) in &mut interactions {
        if matches!(*interaction, Interaction::Pressed) {
            selection.choice = Some(button.kind);
            if let Some(root) = selection.drawer_root.take() {
                despawn_entity_recursive(&mut commands, root, &children_q);
            }
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
