use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::definitions::{BuildCatalog, BuildCategory, BuildDefinitionId};
use super::theme::{paper_panel, shadow_node};
use crate::components::{BuildingMode, GameState, TowerBuildSelection, TowerKind};

#[derive(Resource, Default, Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildMenuState {
    #[default]
    Closed,
    Open,
}

#[derive(Message, Default)]
pub struct ToggleBuildMenu;

#[derive(Component)]
pub struct BuildMenuRoot;

#[derive(Resource, Clone, Copy)]
pub struct CurrentCategory(pub BuildCategory);

impl Default for CurrentCategory {
    fn default() -> Self {
        Self(BuildCategory::Towers)
    }
}

pub fn toggle_build_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<ToggleBuildMenu>,
    game_state: Res<State<GameState>>,
) {
    // Only react in Playing or Paused
    let allow = matches!(game_state.get(), GameState::Playing | GameState::Paused);
    if allow && keyboard.just_pressed(KeyCode::Tab) {
        writer.write(ToggleBuildMenu);
    }
}

pub fn manage_build_menu_ui(
    mut commands: Commands,
    mut state: ResMut<BuildMenuState>,
    mut reader: MessageReader<ToggleBuildMenu>,
    children_q: Query<&Children>,
    roots_q: Query<Entity, With<BuildMenuRoot>>,
    asset_server: Res<AssetServer>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    content_q: Query<Entity, With<BuildContentRoot>>,
    current: Res<CurrentCategory>,
    catalog: Res<BuildCatalog>,
) {
    let mut toggled = false;
    for _ in reader.read() {
        toggled = true;
    }
    if !toggled {
        return;
    }

    *state = match *state {
        BuildMenuState::Closed => {
            next_state.set(GameState::Paused);
            BuildMenuState::Open
        }
        BuildMenuState::Open => {
            next_state.set(GameState::Playing);
            BuildMenuState::Closed
        }
    };

    // Despawn any previous UI
    let menu_roots: Vec<_> = roots_q.iter().collect();
    for e in menu_roots {
        despawn_entity_recursive(&mut commands, e, &children_q);
    }

    if matches!(*state, BuildMenuState::Closed) {
        return;
    }

    // Backdrop
    let backdrop = commands
        .spawn((
            BuildMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
        ))
        .id();

    // Paper panel with simple header; placeholder content for now
    let shadow = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            shadow_node(),
        ))
        .id();

    let panel = commands
        .spawn((paper_panel(), Name::new("BuildPanel")))
        .with_children(|root| {
            // Left category column
            root.spawn((
                Node {
                    width: Val::Px(220.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            ))
            .with_children(|col| {
                let normal_bg = BackgroundColor(Color::srgba(0.98, 0.98, 0.975, 0.9));
                let border = BorderColor::all(Color::srgba(0.18, 0.17, 0.19, 0.9));
                for (cat, label) in [
                    (BuildCategory::Towers, "Towers [1]"),
                    (BuildCategory::Buildings, "Buildings [2]"),
                    (BuildCategory::Plans, "Plans [3]"),
                ] {
                    col.spawn((
                        Button,
                        Node {
                            padding: UiRect::all(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        normal_bg,
                        border,
                        CategoryButton(cat),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(label),
                            TextFont {
                                font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.08, 0.09, 0.11, 1.0)),
                        ));
                    });
                }
            });

            // Right content area placeholder
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.98, 0.98, 0.975, 0.95)),
                BuildContentRoot,
            ))
            .with_children(|_| {});
        })
        .id();

    commands.entity(backdrop).add_child(shadow);
    commands.entity(backdrop).add_child(panel);

    if let Ok(root) = content_q.single() {
        build_grid_under(&mut commands, &asset_server, root, &catalog, current.0);
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

#[derive(Component)]
pub struct CategoryButton(pub BuildCategory);

#[derive(Component)]
pub struct BuildContentRoot;

pub fn handle_category_buttons(
    mut interactions: Query<
        (&Interaction, &CategoryButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut current: ResMut<CurrentCategory>,
    content_root_q: Query<Entity, With<BuildContentRoot>>,
    children_q: Query<&Children>,
    catalog: Res<BuildCatalog>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let mut changed = false;
    for (interaction, category, mut bg) in interactions.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                current.0 = category.0;
                changed = true;
                *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.98));
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgba(0.99, 0.99, 0.985, 0.95));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgba(0.98, 0.98, 0.975, 0.9));
            }
        }
    }

    if !changed {
        return;
    }

    let Ok(root) = content_root_q.single() else {
        return;
    };

    // Clear existing children
    if let Ok(children) = children_q.get(root) {
        for child in children.iter() {
            if commands.get_entity(child).is_ok() {
                despawn_entity_recursive(&mut commands, child, &children_q);
            }
        }
    }

    // Rebuild grid
    build_grid_under(&mut commands, &asset_server, root, &catalog, current.0);
}

fn build_grid_under(
    commands: &mut Commands,
    asset_server: &AssetServer,
    content_root: Entity,
    catalog: &BuildCatalog,
    current: BuildCategory,
) {
    commands.entity(content_root).with_children(|content| {
        content
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(8.0),
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                Name::new("Grid"),
            ))
            .with_children(|grid| {
                for def in catalog.items.iter().filter(|d| d.category == current) {
                    grid.spawn((
                        Button,
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(120.0),
                            padding: UiRect::all(Val::Px(8.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.99, 0.99, 0.985, 0.95)),
                        BorderColor::all(Color::srgba(0.18, 0.17, 0.19, 0.85)),
                        BuildCard(def.id),
                    ))
                    .with_children(|card| {
                        // Icon placeholder (simple square)
                        card.spawn((
                            Node {
                                width: Val::Px(48.0),
                                height: Val::Px(48.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.12, 0.47, 0.95, 0.7)),
                        ));
                        // Name
                        card.spawn((
                            Text::new(def.display_name),
                            TextFont {
                                font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.08, 0.09, 0.11, 1.0)),
                        ));
                        // Cost
                        card.spawn((
                            Text::new(format!("Cost: {}", def.cost)),
                            TextFont {
                                font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.18, 0.17, 0.19, 0.85)),
                        ));
                    });
                }
            });
    });
}

#[derive(Component, Clone, Copy)]
pub struct BuildCard(pub BuildDefinitionId);

pub fn handle_item_selection(
    mut interactions: Query<(&Interaction, &BuildCard), (Changed<Interaction>, With<Button>)>,
    mut selection: ResMut<TowerBuildSelection>,
    mut menu_state: ResMut<BuildMenuState>,
    roots_q: Query<Entity, With<BuildMenuRoot>>,
    children_q: Query<&Children>,
    mut next_state: ResMut<NextState<GameState>>,
    mut building_mode_q: Query<&mut BuildingMode>,
    mut commands: Commands,
) {
    let mut selected: Option<BuildDefinitionId> = None;
    for (interaction, card) in interactions.iter_mut() {
        if matches!(*interaction, Interaction::Pressed) {
            selected = Some(card.0);
            break;
        }
    }
    if let Some(id) = selected {
        let tower_kind = match id.0 {
            "bow_tower" => Some(TowerKind::Bow),
            "crossbow_tower" => Some(TowerKind::Crossbow),
            _ => None,
        };
        if let Some(kind) = tower_kind {
            selection.choice = Some(kind);
            for mut bm in building_mode_q.iter_mut() {
                bm.is_active = true;
            }
        }
        *menu_state = BuildMenuState::Closed;
        next_state.set(GameState::Playing);
        for e in roots_q.iter() {
            if commands.get_entity(e).is_ok() {
                despawn_entity_recursive(&mut commands, e, &children_q);
            }
        }
    }
}
