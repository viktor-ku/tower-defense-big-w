use crate::components::*;
use bevy::input::keyboard::Key;
use bevy::prelude::*;

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
pub struct DrawerSellButton;

pub fn manage_tower_selection_drawer(
    mut commands: Commands,
    building_mode_q: Query<&BuildingMode>,
    mut selection: ResMut<TowerBuildSelection>,
    children_q: Query<&Children>,
    drawer_root_alive: Query<(), With<TowerDrawerRoot>>,
    player_q: Query<&Player>,
    asset_server: Res<AssetServer>,
) {
    let building = building_mode_q.iter().any(|b| b.is_active);

    let need_drawer = building && selection.choice.is_none();
    let has_drawer = selection.drawer_root.is_some();

    if need_drawer && !has_drawer {
        let (player_wood, player_rock) = if let Ok(p) = player_q.single() {
            (p.wood, p.rock)
        } else {
            (0, 0)
        };

        let (bow_wood, bow_rock) = TowerKind::Bow.cost();
        let (xb_wood, xb_rock) = TowerKind::Crossbow.cost();
        let bow_affordable = player_wood >= bow_wood && player_rock >= bow_rock;
        let crossbow_affordable = player_wood >= xb_wood && player_rock >= xb_rock;

        let normal_text = Color::srgba(0.9, 0.92, 0.98, 1.0);
        let disabled_text = Color::srgba(0.7, 0.74, 0.82, 0.7);

        let root = commands
            .spawn((
                TowerDrawerRoot,
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
                // SELL controls at the top of the drawer
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Auto,
                            row_gap: Val::Px(8.0),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.11, 0.16, 0.0)),
                    ))
                    .with_children(|sell| {
                        sell.spawn((
                            Button,
                            DrawerSellButton,
                            Node {
                                padding: UiRect::all(Val::Px(10.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.16, 0.18, 0.25, 0.95)),
                            BorderColor::all(Color::srgba(0.80, 0.55, 0.85, 0.4)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("SELL"),
                                TextFont {
                                    font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.96, 0.92, 1.0, 1.0)),
                            ));
                        });

                        sell.spawn((
                            Text::new("Selling refunds half the spent resources."),
                            TextFont {
                                font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.78, 0.82, 0.9, 0.95)),
                        ));
                    });
                parent.spawn((
                    Text::new("Choose a tower"),
                    TextFont {
                        font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.92, 0.92, 0.98, 1.0)),
                ));
                parent.spawn((
                    Text::new("1 to select Bow, 2 to select Crossbow; Esc to cancel"),
                    TextFont {
                        font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.78, 0.82, 0.9, 1.0)),
                ));

                // Bow option
                {
                    let mut e = parent.spawn((
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
                        p.spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Auto,
                            column_gap: Val::Px(10.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                            .with_children(|row| {
                                row.spawn((
                                    Node {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.35, 0.45, 0.95, 1.0)),
                                ));
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
                                                font: asset_server
                                                    .load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
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
                                            Text::new("Fires quickly but does little damage"),
                                            TextFont {
                                                font: asset_server
                                                    .load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                                                font_size: 16.0,
                                                ..default()
                                            },
                                            TextColor(if bow_affordable {
                                                normal_text
                                            } else {
                                                disabled_text
                                            }),
                                        ));
                                        col.spawn((
                                            Text::new("Range: 30  •  DPS: ~17.1  •  Fire: 0.7s"),
                                            TextFont {
                                                font: asset_server
                                                    .load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                                                font_size: 14.0,
                                                ..default()
                                            },
                                            TextColor(if bow_affordable {
                                                normal_text
                                            } else {
                                                disabled_text
                                            }),
                                        ));
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
                                                    Text::new(format!("{}", bow_wood)),
                                                    TextFont {
                                                        font: asset_server.load(
                                                            "fonts/Nova_Mono/NovaMono-Regular.ttf",
                                                        ),
                                                        font_size: 16.0,
                                                        ..default()
                                                    },
                                                    TextColor(if bow_affordable {
                                                        normal_text
                                                    } else {
                                                        disabled_text
                                                    }),
                                                ));
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
                                                    Text::new(format!("{}", bow_rock)),
                                                    TextFont {
                                                        font: asset_server.load(
                                                            "fonts/Nova_Mono/NovaMono-Regular.ttf",
                                                        ),
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
                                        col.spawn((
                                            Text::new(""),
                                            TextFont {
                                                font: asset_server
                                                    .load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
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
                    let mut e = parent.spawn((
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
                        p.spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Auto,
                            column_gap: Val::Px(10.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                            .with_children(|row| {
                                row.spawn((
                                    Node {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.62, 0.36, 0.86, 1.0)),
                                ));
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
                                                font: asset_server
                                                    .load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
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
                                            Text::new("Fires slowly but does lots of damage"),
                                            TextFont {
                                                font: asset_server
                                                    .load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                                                font_size: 16.0,
                                                ..default()
                                            },
                                            TextColor(if crossbow_affordable {
                                                normal_text
                                            } else {
                                                disabled_text
                                            }),
                                        ));
                                        col.spawn((
                                            Text::new("Range: 30  •  DPS: ~14.6  •  Fire: 2.4s"),
                                            TextFont {
                                                font: asset_server
                                                    .load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                                                font_size: 14.0,
                                                ..default()
                                            },
                                            TextColor(if crossbow_affordable {
                                                normal_text
                                            } else {
                                                disabled_text
                                            }),
                                        ));
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
                                                    Text::new(format!("{}", xb_wood)),
                                                    TextFont {
                                                        font: asset_server.load(
                                                            "fonts/Nova_Mono/NovaMono-Regular.ttf",
                                                        ),
                                                        font_size: 16.0,
                                                        ..default()
                                                    },
                                                    TextColor(if crossbow_affordable {
                                                        normal_text
                                                    } else {
                                                        disabled_text
                                                    }),
                                                ));
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
                                                    Text::new(format!("{}", xb_rock)),
                                                    TextFont {
                                                        font: asset_server.load(
                                                            "fonts/Nova_Mono/NovaMono-Regular.ttf",
                                                        ),
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
                                        col.spawn((
                                            Text::new(""),
                                            TextFont {
                                                font: asset_server
                                                    .load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
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
            })
            .id();
        selection.drawer_root = Some(root);
    } else if !need_drawer
        && has_drawer
        && let Some(root) = selection.drawer_root.take()
        && drawer_root_alive.get(root).is_ok()
    {
        despawn_entity_recursive(&mut commands, root, &children_q);
    }
}

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
        for mut mode in building_mode_q.iter_mut() {
            mode.is_active = false;
        }
        selection.hovered_choice = None;
    }

    if (choose_bow || choose_crossbow || cancel)
        && let Some(root) = selection.drawer_root.take()
    {
        despawn_entity_recursive(&mut commands, root, &children_q);
    }
}

#[allow(clippy::type_complexity)]
pub fn handle_drawer_sell_button_interactions(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<DrawerSellButton>),
    >,
    mut selling_q: Query<&mut SellingMode>,
    mut building_q: Query<&mut BuildingMode>,
    mut selection: ResMut<TowerBuildSelection>,
    children_q: Query<&Children>,
    mut commands: Commands,
) {
    for (interaction, mut bg) in interactions.iter_mut() {
        if matches!(*interaction, Interaction::Pressed) {
            if let Ok(mut selling) = selling_q.single_mut() {
                selling.is_active = true;
            }
            for mut mode in building_q.iter_mut() {
                mode.is_active = false;
            }
            selection.choice = None;
            selection.hovered_choice = None;
            *bg = BackgroundColor(Color::srgba(0.20, 0.12, 0.20, 0.95));

            // Close the drawer immediately
            if let Some(root) = selection.drawer_root.take() {
                despawn_entity_recursive(&mut commands, root, &children_q);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_tower_selection_affordability(
    player_q: Query<&Player>,
    options_q: Query<(Entity, &TowerOption, &Children)>,
    children_q: Query<&Children>,
    mut text_colors: Query<&mut TextColor>,
    mut missing_texts: Query<(&mut Text, &TowerMissingText)>,
    selection: Res<TowerBuildSelection>,
    mut commands: Commands,
) {
    // If the drawer isn't present anymore (e.g., just selected/cancelled this frame),
    // skip updating affordability to avoid issuing commands for entities that will be despawned.
    if selection.drawer_root.is_none() {
        return;
    }
    let Ok(player) = player_q.single() else {
        return;
    };

    let normal_text = Color::srgba(0.9, 0.92, 0.98, 1.0);
    let disabled_text = Color::srgba(0.7, 0.74, 0.82, 0.7);

    for (entity, option, children) in options_q.iter() {
        let (req_wood, req_rock) = option.kind.cost();
        let affordable = player.wood >= req_wood && player.rock >= req_rock;

        if affordable {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.insert((Button, TowerChoiceButton { kind: option.kind }));
            }
        } else {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.remove::<Button>();
                ec.remove::<TowerChoiceButton>();
            }
        }

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

#[allow(clippy::type_complexity)]
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
                *bg = BackgroundColor(Color::srgba(0.18, 0.20, 0.28, 0.95));
                *border = BorderColor::all(Color::srgba(0.75, 0.78, 0.95, 0.55));
                selection.hovered_choice = Some(option.kind);
            }
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgba(0.12, 0.14, 0.20, 0.95));
                *border = BorderColor::all(Color::srgba(0.65, 0.70, 0.85, 0.45));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgba(0.14, 0.16, 0.22, 0.9));
                *border = BorderColor::all(Color::srgba(0.65, 0.70, 0.85, 0.35));
                selection.hovered_choice = None;
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
        if commands.get_entity(entity).is_ok() {
            commands.entity(entity).despawn();
        }
    }
}
