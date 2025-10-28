use crate::components::*;
use bevy::prelude::*;

// Health bar HUD
#[derive(Component)]
pub struct HealthBar;

pub fn spawn_village_health_bar(mut commands: Commands) {
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

pub fn village_health_hud(
    windows: Query<&Window>,
    village_query: Query<&Village, Changed<Village>>,
    mut fill_query: Query<&mut Node, With<HealthBar>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    if let Ok(village) = village_query.single() {
        let health_percentage = village.health as f32 / village.max_health as f32;
        let total_width_px = window.width() * 0.6;
        let fill_width_px = total_width_px * health_percentage.clamp(0.0, 1.0);

        for mut node in fill_query.iter_mut() {
            node.width = Val::Px(fill_width_px);
        }
    }
}

// Resource counters and wave HUD
#[derive(Component)]
pub struct WoodCounterText;

#[derive(Component)]
pub struct RockCounterText;

#[derive(Component)]
pub struct WaveCounterText;

#[derive(Component)]
pub struct WaveTimerText;

#[derive(Component)]
pub(crate) struct ResourceCounter {
    pub(crate) kind: HarvestableKind,
    pub(crate) last_value: u32,
}

#[derive(Component)]
pub(crate) struct WaveCounterDisplay {
    pub(crate) last_value: u32,
}

#[derive(Component)]
pub(crate) struct WaveTimerDisplay {
    pub(crate) last_seconds: Option<u32>,
}

pub fn spawn_resource_counters(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                    font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
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
                    font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
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

pub fn spawn_wave_hud(
    mut commands: Commands,
    wave_state: Res<WaveState>,
    asset_server: Res<AssetServer>,
) {
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
                    font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
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
                    font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgba(0.78, 0.86, 0.95, 1.0)),
                WaveTimerText,
                timer_state,
            ));
        });
}

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

#[allow(clippy::type_complexity)]
pub fn update_wave_hud(
    wave_state: Res<WaveState>,
    mut wave_text_q: Query<(&mut Text, &mut WaveCounterDisplay), With<WaveCounterText>>,
    mut timer_text_q: Query<
        (&mut Text, &mut WaveTimerDisplay),
        (With<WaveTimerText>, Without<WaveCounterText>),
    >,
) {
    if !wave_state.is_changed() {
        return;
    }
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

// Game speed / pause indicator
#[derive(Component)]
pub struct GameSpeedIndicatorText;

pub fn spawn_game_speed_indicator(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            width: Val::Auto,
            height: Val::Auto,
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgba(0.95, 0.95, 0.95, 1.0)),
                GameSpeedIndicatorText,
            ));
        });
}

pub fn update_game_speed_indicator(
    state: Res<State<GameState>>,
    mut query: Query<&mut Text, With<GameSpeedIndicatorText>>,
) {
    let desired = match state.get() {
        GameState::Playing => "1x",
        GameState::Paused => "||",
        _ => "",
    };
    for mut text in query.iter_mut() {
        *text = Text::new(desired.to_string());
    }
}

// (SELL button moved into the tower drawer)
