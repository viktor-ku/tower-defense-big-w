use bevy::asset::LoadedUntypedAsset;
use bevy::audio::AudioSource;
use bevy::prelude::*;
use bevy::ui::widget::ImageNode;

use crate::components::GameState;

#[derive(Component)]
struct SplashRoot;

#[derive(Resource, Default)]
struct LoadingAssets {
    // Core assets we want ready before gameplay
    audio: Handle<AudioSource>,
    font: Handle<Font>,
    logo: Handle<Image>,
    shaders: Vec<Handle<LoadedUntypedAsset>>,
}

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), on_enter_loading)
            .add_systems(
                Update,
                (queue_preloads, check_preloads).run_if(in_state(GameState::Loading)),
            )
            .add_systems(OnExit(GameState::Loading), on_exit_loading);
    }
}

#[derive(Resource)]
struct LoadingDelay(Timer);

fn on_enter_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera for splash UI
    commands.spawn((Camera2d, SplashRoot));

    // Ensure splash stays up for at least 2s
    commands.insert_resource(LoadingDelay(Timer::from_seconds(2.0, TimerMode::Once)));

    // Fullscreen centered column (logo + text)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            SplashRoot,
        ))
        .with_children(|parent| {
            // Logo image
            parent.spawn((
                Node {
                    width: Val::Px(384.0),
                    height: Val::Px(384.0),
                    ..default()
                },
                ImageNode::new(asset_server.load("images/logo-512x.png")),
            ));

            // Loading text
            parent.spawn((
                Text::new("Loading..."),
                TextFont {
                    font: asset_server.load("fonts/Luckiest_Guy/LuckiestGuy-Regular.ttf"),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.98)),
            ));
        });
}

fn queue_preloads(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    maybe: Option<Res<LoadingAssets>>,
) {
    if maybe.is_some() {
        return;
    }
    // Begin preloading core assets (extend as needed)
    let audio: Handle<AudioSource> = asset_server.load("sounds/round-start.wav");
    let font: Handle<Font> = asset_server.load("fonts/Luckiest_Guy/LuckiestGuy-Regular.ttf");
    let logo: Handle<Image> = asset_server.load("images/logo-512x.png");

    let shaders = vec![
        asset_server.load_untyped("shaders/projectile.wgsl"),
        asset_server.load_untyped("shaders/impact.wgsl"),
        asset_server.load_untyped("shaders/explosion.wgsl"),
        asset_server.load_untyped("shaders/trail.wgsl"),
    ];

    commands.insert_resource(LoadingAssets {
        audio,
        font,
        logo,
        shaders,
    });
}

fn check_preloads(
    asset_server: Res<AssetServer>,
    assets: Option<Res<LoadingAssets>>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut delay: ResMut<LoadingDelay>,
) {
    let Some(assets) = assets else {
        return;
    };

    // If all assets (including dependencies) are loaded, proceed to Playing
    let shaders_ready = assets
        .shaders
        .iter()
        .all(|h| asset_server.is_loaded_with_dependencies(h.id()));
    let audio_ready = asset_server.is_loaded_with_dependencies(assets.audio.id());
    let font_ready = asset_server.is_loaded_with_dependencies(assets.font.id());
    let logo_ready = asset_server.is_loaded_with_dependencies(assets.logo.id());

    // Tick the minimum display timer
    delay.0.tick(time.delta());

    if shaders_ready && audio_ready && font_ready && logo_ready && delay.0.is_finished() {
        next_state.set(GameState::Playing);
    }
}

fn on_exit_loading(
    mut commands: Commands,
    roots: Query<Entity, With<SplashRoot>>,
    children_q: Query<&Children>,
) {
    fn despawn_tree(entity: Entity, commands: &mut Commands, children_q: &Query<&Children>) {
        if let Ok(children) = children_q.get(entity) {
            for child in children.iter() {
                despawn_tree(child, commands, children_q);
            }
        }
        commands.entity(entity).despawn();
    }

    for e in roots.iter() {
        despawn_tree(e, &mut commands, &children_q);
    }
}
