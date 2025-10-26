use crate::components::*;
use crate::events::*;
use bevy::input::keyboard::Key;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

// Observer-based logging for gameplay events (Bevy 0.17)
pub fn on_resource_collected(trigger: On<ResourceCollected>) {
    let e = trigger.event();
    if cfg!(debug_assertions) {
        info!("Resource collected: {:?} x{}", e.kind, e.amount);
    }
}

pub fn on_tower_built(trigger: On<TowerBuilt>) {
    let e = trigger.event();
    if cfg!(debug_assertions) {
        info!("Tower built at: {:?}", e.position);
    }
}

pub fn on_enemy_spawned(trigger: On<EnemySpawned>) {
    let e = trigger.event();
    if cfg!(debug_assertions) {
        info!("Enemy spawned at: {:?}", e.position);
    }
}

pub fn on_enemy_killed(trigger: On<EnemyKilled>) {
    let e = trigger.event();
    if cfg!(debug_assertions) {
        info!("Enemy killed at: {:?}", e.position);
    }
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

#[derive(Component)]
pub struct TowerOption {
    pub kind: TowerKind,
}

#[derive(Component)]
pub struct TowerMissingText {
    pub kind: TowerKind,
}

#[derive(Component)]
pub struct TowerDrawerViewport;

#[derive(Component)]
pub struct TowerListContent {
    pub offset_px: f32,
}

#[derive(Component)]
pub struct TowerScrollbarThumb;

/// Spawns a right-side drawer prompting the player to choose a tower when in build mode
/// and no selection is currently chosen. Despawns it otherwise.
pub fn manage_tower_selection_drawer(
    mut commands: Commands,
    building_mode_q: Query<&BuildingMode>,
    mut selection: ResMut<TowerBuildSelection>,
    children_q: Query<&Children>,
    drawer_root_alive: Query<(), With<TowerDrawerRoot>>,
    player_q: Query<&Player>,
) {
    let building = building_mode_q.iter().any(|b| b.is_active);

    let need_drawer = building && selection.choice.is_none();
    let has_drawer = selection.drawer_root.is_some();

    if need_drawer && !has_drawer {
        // Spawn drawer root on the right side
        // Check current resources for affordability
        let (player_wood, player_rock) = if let Ok(p) = player_q.single() {
            (p.wood, p.rock)
        } else {
            (0, 0)
        };

        let bow_affordable = player_wood >= 3 && player_rock >= 1;
        let crossbow_affordable = player_wood >= 10 && player_rock >= 3;

        let normal_text = Color::srgba(0.9, 0.92, 0.98, 1.0);
        let disabled_text = Color::srgba(0.7, 0.74, 0.82, 0.7);

        let root = commands
            .spawn((
                TowerDrawerRoot,
                TowerDrawerViewport,
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(360.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    row_gap: Val::Px(10.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip(),
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
                parent.spawn((
                    Text::new("1 to select Bow, 2 to select Crossbow; Esc to cancel"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.78, 0.82, 0.9, 1.0)),
                ));

                // Scrollable content wrapper
                parent
                    .spawn((
                        TowerListContent { offset_px: 0.0 },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Auto,
                            top: Val::Px(0.0),
                            row_gap: Val::Px(10.0),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                    ))
                    .with_children(|list| {
                        // Bow option
                        {
                            let mut e = list.spawn((
                                TowerOption {
                                    kind: TowerKind::Bow,
                                },
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Auto,
                                    padding: UiRect::all(Val::Px(14.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    row_gap: Val::Px(4.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.14, 0.16, 0.22, 0.9)),
                                BorderColor::all(Color::srgba(0.65, 0.70, 0.85, 0.35)),
                            ));
                            if bow_affordable {
                                e.insert((
                                    Button,
                                    TowerChoiceButton {
                                        kind: TowerKind::Bow,
                                    },
                                ));
                            }
                            e.with_children(|p| {
                                // Row container: icon + text column
                                p.spawn((Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Auto,
                                    column_gap: Val::Px(10.0),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },))
                                    .with_children(|row| {
                                        // Left icon (blue-ish for Bow)
                                        row.spawn((
                                            Node {
                                                width: Val::Px(24.0),
                                                height: Val::Px(24.0),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(0.35, 0.45, 0.95, 1.0)),
                                        ));

                                        // Text column
                                        row.spawn((Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Auto,
                                            row_gap: Val::Px(2.0),
                                            flex_direction: FlexDirection::Column,
                                            ..default()
                                        },))
                                            .with_children(|col| {
                                                col.spawn((
                                                    Text::new("Bow tower [1]"),
                                                    TextFont {
                                                        font_size: 20.0,
                                                        ..default()
                                                    },
                                                    TextColor(if bow_affordable {
                                                        normal_text
                                                    } else {
                                                        disabled_text
                                                    }),
                                                ));
                                                col.spawn((
                                                    Text::new(
                                                        "Fires quickly but does little damage",
                                                    ),
                                                    TextFont {
                                                        font_size: 16.0,
                                                        ..default()
                                                    },
                                                    TextColor(if bow_affordable {
                                                        normal_text
                                                    } else {
                                                        disabled_text
                                                    }),
                                                ));
                                                // Stats line
                                                col.spawn((
                                                    Text::new(
                                                        "Range: 30  •  DPS: ~17.1  •  Fire: 0.7s",
                                                    ),
                                                    TextFont {
                                                        font_size: 14.0,
                                                        ..default()
                                                    },
                                                    TextColor(if bow_affordable {
                                                        normal_text
                                                    } else {
                                                        disabled_text
                                                    }),
                                                ));
                                                // Cost row with icons
                                                col.spawn((Node {
                                                    width: Val::Percent(100.0),
                                                    height: Val::Auto,
                                                    column_gap: Val::Px(8.0),
                                                    flex_direction: FlexDirection::Row,
                                                    justify_content: JustifyContent::FlexEnd,
                                                    align_items: AlignItems::Center,
                                                    ..default()
                                                },))
                                                    .with_children(|cost| {
                                                        // Wood icon
                                                        cost.spawn((
                                                            Node {
                                                                width: Val::Px(10.0),
                                                                height: Val::Px(10.0),
                                                                ..default()
                                                            },
                                                            BackgroundColor(Color::srgba(
                                                                0.93, 0.86, 0.68, 1.0,
                                                            )),
                                                        ));
                                                        cost.spawn((
                                                            Text::new("3"),
                                                            TextFont {
                                                                font_size: 16.0,
                                                                ..default()
                                                            },
                                                            TextColor(if bow_affordable {
                                                                normal_text
                                                            } else {
                                                                disabled_text
                                                            }),
                                                        ));
                                                        // Rock icon
                                                        cost.spawn((
                                                            Node {
                                                                width: Val::Px(10.0),
                                                                height: Val::Px(10.0),
                                                                ..default()
                                                            },
                                                            BackgroundColor(Color::srgba(
                                                                0.86, 0.88, 0.95, 1.0,
                                                            )),
                                                        ));
                                                        cost.spawn((
                                                            Text::new("1"),
                                                            TextFont {
                                                                font_size: 16.0,
                                                                ..default()
                                                            },
                                                            TextColor(if bow_affordable {
                                                                normal_text
                                                            } else {
                                                                disabled_text
                                                            }),
                                                        ));
                                                    });
                                                // Missing resources line
                                                col.spawn((
                                                    Text::new(""),
                                                    TextFont {
                                                        font_size: 14.0,
                                                        ..default()
                                                    },
                                                    TextColor(Color::srgba(0.86, 0.5, 0.5, 0.9)),
                                                    TowerMissingText {
                                                        kind: TowerKind::Bow,
                                                    },
                                                ));
                                            });
                                    });
                            });
                        }

                        // Crossbow option
                        {
                            let mut e = list.spawn((
                                TowerOption {
                                    kind: TowerKind::Crossbow,
                                },
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Auto,
                                    padding: UiRect::all(Val::Px(14.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    row_gap: Val::Px(4.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.14, 0.16, 0.22, 0.9)),
                                BorderColor::all(Color::srgba(0.65, 0.70, 0.85, 0.35)),
                            ));
                            if crossbow_affordable {
                                e.insert((
                                    Button,
                                    TowerChoiceButton {
                                        kind: TowerKind::Crossbow,
                                    },
                                ));
                            }
                            e.with_children(|p| {
                                // Row container: icon + text column
                                p.spawn((Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Auto,
                                    column_gap: Val::Px(10.0),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },))
                                    .with_children(|row| {
                                        // Left icon (purple for Crossbow)
                                        row.spawn((
                                            Node {
                                                width: Val::Px(24.0),
                                                height: Val::Px(24.0),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(0.62, 0.36, 0.86, 1.0)),
                                        ));

                                        // Text column
                                        row.spawn((Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Auto,
                                            row_gap: Val::Px(2.0),
                                            flex_direction: FlexDirection::Column,
                                            ..default()
                                        },))
                                            .with_children(|col| {
                                                col.spawn((
                                                    Text::new("Crossbow tower [2]"),
                                                    TextFont {
                                                        font_size: 20.0,
                                                        ..default()
                                                    },
                                                    TextColor(if crossbow_affordable {
                                                        normal_text
                                                    } else {
                                                        disabled_text
                                                    }),
                                                ));
                                                col.spawn((
                                                    Text::new(
                                                        "Fires slowly but does lots of damage",
                                                    ),
                                                    TextFont {
                                                        font_size: 16.0,
                                                        ..default()
                                                    },
                                                    TextColor(if crossbow_affordable {
                                                        normal_text
                                                    } else {
                                                        disabled_text
                                                    }),
                                                ));
                                                // Stats line
                                                col.spawn((
                                                    Text::new(
                                                        "Range: 30  •  DPS: ~14.6  •  Fire: 2.4s",
                                                    ),
                                                    TextFont {
                                                        font_size: 14.0,
                                                        ..default()
                                                    },
                                                    TextColor(if crossbow_affordable {
                                                        normal_text
                                                    } else {
                                                        disabled_text
                                                    }),
                                                ));
                                                // Cost row with icons
                                                col.spawn((Node {
                                                    width: Val::Percent(100.0),
                                                    height: Val::Auto,
                                                    column_gap: Val::Px(8.0),
                                                    flex_direction: FlexDirection::Row,
                                                    justify_content: JustifyContent::FlexEnd,
                                                    align_items: AlignItems::Center,
                                                    ..default()
                                                },))
                                                    .with_children(|cost| {
                                                        // Wood icon
                                                        cost.spawn((
                                                            Node {
                                                                width: Val::Px(10.0),
                                                                height: Val::Px(10.0),
                                                                ..default()
                                                            },
                                                            BackgroundColor(Color::srgba(
                                                                0.93, 0.86, 0.68, 1.0,
                                                            )),
                                                        ));
                                                        cost.spawn((
                                                            Text::new("10"),
                                                            TextFont {
                                                                font_size: 16.0,
                                                                ..default()
                                                            },
                                                            TextColor(if crossbow_affordable {
                                                                normal_text
                                                            } else {
                                                                disabled_text
                                                            }),
                                                        ));
                                                        // Rock icon
                                                        cost.spawn((
                                                            Node {
                                                                width: Val::Px(10.0),
                                                                height: Val::Px(10.0),
                                                                ..default()
                                                            },
                                                            BackgroundColor(Color::srgba(
                                                                0.86, 0.88, 0.95, 1.0,
                                                            )),
                                                        ));
                                                        cost.spawn((
                                                            Text::new("3"),
                                                            TextFont {
                                                                font_size: 16.0,
                                                                ..default()
                                                            },
                                                            TextColor(if crossbow_affordable {
                                                                normal_text
                                                            } else {
                                                                disabled_text
                                                            }),
                                                        ));
                                                    });
                                                // Missing resources line
                                                col.spawn((
                                                    Text::new(""),
                                                    TextFont {
                                                        font_size: 14.0,
                                                        ..default()
                                                    },
                                                    TextColor(Color::srgba(0.86, 0.5, 0.5, 0.9)),
                                                    TowerMissingText {
                                                        kind: TowerKind::Crossbow,
                                                    },
                                                ));
                                            });
                                    });
                            });
                        }
                    });
            })
            .with_children(|parent| {
                // Slim scrollbar thumb overlay
                parent.spawn((
                    TowerScrollbarThumb,
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(2.0),
                        top: Val::Px(2.0),
                        width: Val::Px(4.0),
                        height: Val::Px(40.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.75, 0.78, 0.95, 0.45)),
                ));
            })
            .id();
        selection.drawer_root = Some(root);
    } else if !need_drawer
        && has_drawer
        && let Some(root) = selection.drawer_root.take()
    {
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
            selection.hovered_choice = None;
            if let Some(root) = selection.drawer_root.take() {
                despawn_entity_recursive(&mut commands, root, &children_q);
            }
        }
    }
}

/// Keyboard shortcuts for the tower drawer: 1=Bow, 2=Crossbow, Esc=cancel build mode
pub fn tower_drawer_shortcuts(
    keyboard_input: Res<ButtonInput<Key>>,
    mut selection: ResMut<TowerBuildSelection>,
    mut building_mode_q: Query<&mut BuildingMode>,
    children_q: Query<&Children>,
    mut commands: Commands,
) {
    if selection.drawer_root.is_none() {
        return;
    }

    let choose_bow = keyboard_input.just_pressed(Key::Character("1".into()));
    let choose_crossbow = keyboard_input.just_pressed(Key::Character("2".into()));
    let cancel = keyboard_input.just_pressed(Key::Escape);

    if choose_bow {
        selection.choice = Some(TowerKind::Bow);
    } else if choose_crossbow {
        selection.choice = Some(TowerKind::Crossbow);
    } else if cancel {
        // Turn off building mode to cancel drawer
        for mut mode in building_mode_q.iter_mut() {
            mode.is_active = false;
        }
        selection.hovered_choice = None;
    }

    // Close drawer if a choice was made or cancelled
    if choose_bow || choose_crossbow || cancel {
        if let Some(root) = selection.drawer_root.take() {
            despawn_entity_recursive(&mut commands, root, &children_q);
        }
    }
}

/// Live-updates the tower options to reflect the player's current resources.
pub fn update_tower_selection_affordability(
    player_q: Query<&Player>,
    options_q: Query<(Entity, &TowerOption, &Children)>,
    children_q: Query<&Children>,
    mut text_colors: Query<&mut TextColor>,
    mut missing_texts: Query<(&mut Text, &TowerMissingText)>,
    mut commands: Commands,
) {
    let Ok(player) = player_q.single() else {
        return;
    };

    let normal_text = Color::srgba(0.9, 0.92, 0.98, 1.0);
    let disabled_text = Color::srgba(0.7, 0.74, 0.82, 0.7);

    for (entity, option, children) in options_q.iter() {
        let (req_wood, req_rock) = match option.kind {
            TowerKind::Bow => (3, 1),
            TowerKind::Crossbow => (10, 3),
        };
        let affordable = player.wood >= req_wood && player.rock >= req_rock;

        // Toggle clickability
        if affordable {
            commands
                .entity(entity)
                .insert((Button, TowerChoiceButton { kind: option.kind }));
        } else {
            commands
                .entity(entity)
                .remove::<Button>()
                .remove::<TowerChoiceButton>();
        }

        // Update text colors recursively across descendants
        let color = if affordable {
            normal_text
        } else {
            disabled_text
        };
        let mut stack: Vec<Entity> = Vec::new();
        for c in children.iter() {
            stack.push(c);
        }
        while let Some(e) = stack.pop() {
            if let Ok(mut tc) = text_colors.get_mut(e) {
                *tc = TextColor(color);
            }
            if let Ok(grand) = children_q.get(e) {
                for g in grand.iter() {
                    stack.push(g);
                }
            }
        }

        // Update missing-resource line for this tower kind
        let need_wood = req_wood.saturating_sub(player.wood);
        let need_rock = req_rock.saturating_sub(player.rock);
        for (mut text, tag) in missing_texts.iter_mut() {
            if tag.kind != option.kind {
                continue;
            }
            if affordable {
                *text = Text::new("");
            } else {
                let mut msg = String::from("(need ");
                let mut first = true;
                if need_wood > 0 {
                    msg.push_str(&format!("+{} wood", need_wood));
                    first = false;
                }
                if need_rock > 0 {
                    if !first {
                        msg.push_str(", ");
                    }
                    msg.push_str(&format!("+{} rock", need_rock));
                }
                msg.push(')');
                *text = Text::new(msg);
            }
        }
    }
}

/// Subtle hover effect for affordable tower options (only those with `Button`).
pub fn update_tower_option_hover(
    mut q: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &TowerOption,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut selection: ResMut<TowerBuildSelection>,
) {
    for (interaction, mut bg, mut border, option) in q.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                // Slightly brighten on hover
                *bg = BackgroundColor(Color::srgba(0.18, 0.20, 0.28, 0.95));
                *border = BorderColor::all(Color::srgba(0.75, 0.78, 0.95, 0.55));
                // Set hovered preview kind
                selection.hovered_choice = Some(option.kind);
            }
            Interaction::Pressed => {
                // Subtle pressed feedback
                *bg = BackgroundColor(Color::srgba(0.12, 0.14, 0.20, 0.95));
                *border = BorderColor::all(Color::srgba(0.65, 0.70, 0.85, 0.45));
            }
            Interaction::None => {
                // Return to default card color
                *bg = BackgroundColor(Color::srgba(0.14, 0.16, 0.22, 0.9));
                *border = BorderColor::all(Color::srgba(0.65, 0.70, 0.85, 0.35));
                // Clear hovered preview when not interacting
                selection.hovered_choice = None;
            }
        }
    }
}

/// Mouse wheel scroll for the full-height drawer viewport
pub fn tower_drawer_scroll(
    mut wheel: MessageReader<MouseWheel>,
    viewport_q: Query<&Interaction, With<TowerDrawerViewport>>,
    mut content_q: Query<(&mut Node, &mut TowerListContent), With<TowerListContent>>,
) {
    // Only scroll when the drawer is hovered
    let hovered = viewport_q
        .iter()
        .any(|i| matches!(*i, Interaction::Hovered | Interaction::Pressed));
    if !hovered {
        // Drain events so they don't accumulate
        for _ in wheel.read() {}
        return;
    }

    let mut delta = 0.0f32;
    for ev in wheel.read() {
        // Approximate line height step for Line unit; Pixel as-is
        use bevy::input::mouse::MouseScrollUnit;
        delta += match ev.unit {
            MouseScrollUnit::Line => ev.y * 24.0,
            MouseScrollUnit::Pixel => ev.y,
        } as f32;
    }

    if delta.abs() < f32::EPSILON {
        return;
    }

    if let Ok((mut node, mut state)) = content_q.single_mut() {
        // Positive delta usually means scroll up; decrease offset
        state.offset_px = (state.offset_px - delta).max(0.0);
        node.top = Val::Px(-state.offset_px);
    }
}

/// Updates scrollbar thumb size and position based on estimated content size and current offset
pub fn update_tower_scrollbar_thumb(
    windows: Query<&Window>,
    content_state_q: Query<&TowerListContent, With<TowerDrawerRoot>>,
    options_q: Query<&TowerOption>,
    mut thumb_q: Query<&mut Node, With<TowerScrollbarThumb>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(state) = content_state_q.single() else {
        return;
    };
    let Ok(mut thumb) = thumb_q.single_mut() else {
        return;
    };

    // Estimate content height from item count
    let items = options_q.iter().count().max(1);
    let est_item_h = 140.0f32; // approximate card height with padding and lines
    let est_gap = 10.0f32;
    let content_h = items as f32 * est_item_h + (items.saturating_sub(1) as f32) * est_gap;

    // Viewport height: drawer is full-height with 14px top/bottom padding
    let viewport_h = (window.height() - 28.0).max(0.0);
    if viewport_h <= 0.0 || content_h <= 0.0 {
        return;
    }

    let max_offset = (content_h - viewport_h).max(0.0);
    let offset = state.offset_px.clamp(0.0, max_offset);

    // Thumb height proportional to viewport/content; clamp to a minimum
    let min_thumb = 24.0;
    let mut thumb_h = (viewport_h / content_h) * viewport_h;
    if thumb_h < min_thumb {
        thumb_h = min_thumb;
    }
    if thumb_h > viewport_h {
        thumb_h = viewport_h;
    }

    // Thumb top position
    let track_h = viewport_h - thumb_h;
    let thumb_top = if max_offset > 0.0 {
        (offset / max_offset) * track_h
    } else {
        0.0
    };

    thumb.top = Val::Px(thumb_top + 2.0);
    thumb.height = Val::Px(thumb_h.max(0.0));
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
