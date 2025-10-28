use bevy::prelude::*;
use bevy_kira_audio::AudioSource as KiraAudioSource;
use bevy_kira_audio::prelude::*;
pub mod sfx;
pub mod util;

// Marker types for logical audio channels
#[derive(Resource)]
pub struct SfxChannel;
#[derive(Resource)]
pub struct UiChannel;
#[derive(Resource)]
pub struct MusicChannel;
#[derive(Resource)]
pub struct AmbienceChannel;

// Global volume controls per category
#[derive(Resource, Clone, Copy)]
pub struct AudioVolumes {
    pub master: f32,
    pub sfx: f32,
    pub music: f32,
    pub ui: f32,
    pub ambience: f32,
}

impl Default for AudioVolumes {
    fn default() -> Self {
        Self {
            master: 1.0,
            sfx: 1.0,
            music: 1.0,
            ui: 1.0,
            ambience: 1.0,
        }
    }
}

// Events/messages that other gameplay systems can emit
#[derive(Debug, Clone, Copy)]
pub enum TowerShotKind {
    Bow,
    Crossbow,
}

#[derive(Event, Message, Debug, Clone, Copy)]
pub struct TowerShotEvent {
    pub kind: TowerShotKind,
    pub position: Vec3,
}

#[derive(Event, Message, Debug, Clone, Copy)]
pub struct PlayerFootstepEvent {
    pub position: Vec3,
}

#[derive(Event, Message, Debug, Clone, Copy)]
pub struct WaveStartedEvent;

#[derive(Event, Message, Debug, Clone, Copy)]
pub struct BossWaveStartedEvent;

#[derive(Debug, Clone, Copy)]
pub enum BuildingActionKind {
    Place,
    Invalid,
    Upgrade,
    Sell,
}

#[derive(Event, Message, Debug, Clone, Copy)]
pub struct BuildingActionEvent {
    pub kind: BuildingActionKind,
    pub position: Vec3,
}

// Centralized handles to audio assets we care about
#[derive(Resource, Default)]
pub struct AudioAssets {
    pub tower_bow_release: Handle<KiraAudioSource>,
    pub tower_crossbow_release: Handle<KiraAudioSource>,
    pub wave_start: Handle<KiraAudioSource>,
    pub wave_start_boss: Handle<KiraAudioSource>,
    pub player_footstep_01: Handle<KiraAudioSource>,
    pub tower_place: Handle<KiraAudioSource>,
    pub tower_place_invalid: Handle<KiraAudioSource>,
    pub tower_upgrade: Handle<KiraAudioSource>,
    pub tower_sell: Handle<KiraAudioSource>,
}

// Marker placed on the active camera used as audio listener
#[derive(Component)]
pub struct AudioListener;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            // Logical audio channels
            .add_audio_channel::<SfxChannel>()
            .add_audio_channel::<UiChannel>()
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<AmbienceChannel>()
            // Volumes and assets
            .init_resource::<AudioVolumes>()
            .init_resource::<AudioAssets>()
            .init_resource::<SpatialAudioParams>()
            .init_resource::<ListenerTransform>()
            // Messages for audio-triggering events
            .add_message::<TowerShotEvent>()
            .add_message::<PlayerFootstepEvent>()
            .add_message::<WaveStartedEvent>()
            .add_message::<BossWaveStartedEvent>()
            .add_message::<BuildingActionEvent>()
            // Keep listener transform updated each frame
            .add_systems(Update, update_listener_transform)
            // Load audio handles at startup
            .add_systems(Startup, load_audio_assets)
            // Observers to react to gameplay messages
            .add_observer(on_tower_shot)
            .add_observer(on_player_footstep)
            .add_observer(on_wave_started)
            .add_observer(on_boss_wave_started)
            .add_observer(on_building_action);
    }
}

// Parameters for simple 2D-ish spatialization in a top-down game
#[derive(Resource, Clone, Copy)]
pub struct SpatialAudioParams {
    pub attenuation: f32,          // higher => faster volume falloff
    pub max_audible_distance: f32, // hard clamp to mute beyond this
}

impl Default for SpatialAudioParams {
    fn default() -> Self {
        Self {
            attenuation: 0.08,
            max_audible_distance: 80.0,
        }
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub struct ListenerTransform(pub Option<GlobalTransform>);

fn update_listener_transform(
    q_listener: Query<&GlobalTransform, (With<Camera>, With<AudioListener>)>,
    mut listener_tf: ResMut<ListenerTransform>,
) {
    listener_tf.0 = q_listener.iter().next().copied();
}

pub fn spatialize(
    source_world: Vec3,
    listener: &GlobalTransform,
    params: SpatialAudioParams,
) -> (f32, f32) {
    let listener_translation = listener.translation();
    let to_source = source_world - listener_translation;
    let distance = to_source.length();
    let volume = if distance >= params.max_audible_distance {
        0.0
    } else {
        (1.0 / (1.0 + params.attenuation * distance)).clamp(0.0, 1.0)
    };

    let listener_tr = listener.compute_transform();
    let right = listener_tr.rotation * Vec3::X;
    let dir_norm = if distance > 0.0001 {
        to_source / distance
    } else {
        Vec3::ZERO
    };
    let pan = dir_norm.dot(right).clamp(-1.0, 1.0);
    (volume, pan)
}

fn load_audio_assets(asset_server: Res<AssetServer>, mut assets: ResMut<AudioAssets>) {
    // Note: loaders pick the first existing extension in assets/audio/sfx.
    assets.tower_bow_release = sfx::tower_bow_release::load(&asset_server);
    assets.tower_crossbow_release = sfx::tower_crossbow_release::load(&asset_server);
    assets.wave_start = sfx::wave_start::load(&asset_server);
    assets.wave_start_boss = sfx::wave_start_boss::load(&asset_server);
    assets.player_footstep_01 = sfx::player_footstep_01::load(&asset_server);
    assets.tower_place = sfx::tower_place::load(&asset_server);
    assets.tower_place_invalid = sfx::tower_place_invalid::load(&asset_server);
    assets.tower_upgrade = sfx::tower_upgrade::load(&asset_server);
    assets.tower_sell = sfx::tower_sell::load(&asset_server);
}

fn effective_sfx_volume(volumes: &AudioVolumes) -> f32 {
    (volumes.master * volumes.sfx).clamp(0.0, 1.0)
}

pub fn on_tower_shot(
    trigger: On<TowerShotEvent>,
    sfx: Res<AudioChannel<SfxChannel>>,
    assets: Res<AudioAssets>,
    volumes: Res<AudioVolumes>,
    params: Res<SpatialAudioParams>,
    listener_tf: Res<ListenerTransform>,
) {
    let e = trigger.event();
    let listener = listener_tf.0.unwrap_or(GlobalTransform::IDENTITY);
    let (vol, pan) = spatialize(e.position, &listener, *params);
    let base = effective_sfx_volume(&volumes);
    let handle = match e.kind {
        TowerShotKind::Bow => assets.tower_bow_release.clone(),
        TowerShotKind::Crossbow => assets.tower_crossbow_release.clone(),
    };
    sfx.play(handle).with_volume(base * vol).with_panning(pan);
}

pub fn on_player_footstep(
    trigger: On<PlayerFootstepEvent>,
    sfx: Res<AudioChannel<SfxChannel>>,
    assets: Res<AudioAssets>,
    volumes: Res<AudioVolumes>,
    params: Res<SpatialAudioParams>,
    listener_tf: Res<ListenerTransform>,
) {
    let e = trigger.event();
    let listener = listener_tf.0.unwrap_or(GlobalTransform::IDENTITY);
    let (vol, pan) = spatialize(e.position, &listener, *params);
    let base = effective_sfx_volume(&volumes);
    sfx.play(assets.player_footstep_01.clone())
        .with_volume(base * vol)
        .with_panning(pan);
}

pub fn on_wave_started(
    _trigger: On<WaveStartedEvent>,
    sfx: Res<AudioChannel<SfxChannel>>,
    assets: Res<AudioAssets>,
    volumes: Res<AudioVolumes>,
) {
    let base = effective_sfx_volume(&volumes);
    sfx.play(assets.wave_start.clone()).with_volume(base);
}

pub fn on_boss_wave_started(
    _trigger: On<BossWaveStartedEvent>,
    sfx: Res<AudioChannel<SfxChannel>>,
    assets: Res<AudioAssets>,
    volumes: Res<AudioVolumes>,
) {
    let base = effective_sfx_volume(&volumes);
    sfx.play(assets.wave_start_boss.clone()).with_volume(base);
}

pub fn on_building_action(
    trigger: On<BuildingActionEvent>,
    sfx: Res<AudioChannel<SfxChannel>>,
    assets: Res<AudioAssets>,
    volumes: Res<AudioVolumes>,
    params: Res<SpatialAudioParams>,
    listener_tf: Res<ListenerTransform>,
) {
    let e = trigger.event();
    let listener = listener_tf.0.unwrap_or(GlobalTransform::IDENTITY);
    let (vol, pan) = spatialize(e.position, &listener, *params);
    let base = effective_sfx_volume(&volumes);
    let handle = match e.kind {
        BuildingActionKind::Place => assets.tower_place.clone(),
        BuildingActionKind::Invalid => assets.tower_place_invalid.clone(),
        BuildingActionKind::Upgrade => assets.tower_upgrade.clone(),
        BuildingActionKind::Sell => assets.tower_sell.clone(),
    };
    sfx.play(handle).with_volume(base * vol).with_panning(pan);
}
