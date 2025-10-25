use crate::components::{ChunkRoot, Harvestable, HarvestableKind, Player, Tree};
use crate::constants::Tunables;
use bevy::prelude::*;
// UI debug overlay omitted for now; logging is used instead
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl Default for ChunkCoord {
    fn default() -> Self {
        ChunkCoord { x: 0, z: 0 }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct WorldSeed(pub u64);

#[derive(Resource, Clone, Copy)]
pub struct ChunkConfig {
    pub size: f32,
    pub active_radius: i32,
    pub hysteresis: i32,
    pub max_loads_per_frame: usize,
    pub max_unloads_per_frame: usize,
}

#[derive(Resource, Default)]
pub struct LoadedChunks(pub HashMap<ChunkCoord, Entity>);

#[derive(Resource, Default, Clone, Copy)]
pub struct PlayerChunk(pub ChunkCoord);

#[derive(Resource, Default)]
pub struct ChunkAssets {
    pub tree_mesh: Handle<Mesh>,
    pub tree_mat: Handle<StandardMaterial>,
    pub rock_mesh: Handle<Mesh>,
    pub rock_mat: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct ChunkHudState {
    pub enabled: bool,
    pub root: Option<Entity>,
}

#[derive(Component)]
struct ChunkHudRoot;

#[derive(Component)]
struct ChunkHudText;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        // Initialize chunking config from Tunables at plugin build time
        if let Some(tunables) = app.world().get_resource::<Tunables>().cloned() {
            app.insert_resource(WorldSeed(tunables.world_seed));
            app.insert_resource(ChunkConfig {
                size: tunables.chunk_size,
                active_radius: tunables.chunks_active_radius,
                hysteresis: tunables.chunks_hysteresis,
                max_loads_per_frame: tunables.chunks_loads_per_frame,
                max_unloads_per_frame: tunables.chunks_unloads_per_frame,
            });
        }

        app.init_resource::<LoadedChunks>()
            .insert_resource(PlayerChunk(ChunkCoord { x: 0, z: 0 }))
            .insert_resource(ChunkHudState {
                enabled: true,
                root: None,
            })
            .add_systems(Startup, (setup_chunk_assets,))
            .add_systems(
                Update,
                (
                    track_player_chunk,
                    update_chunks,
                    chunk_hud_toggle,
                    update_chunk_hud_text,
                    chunk_config_shortcuts,
                    distance_culling,
                ),
            );
    }
}

fn setup_chunk_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tunables: Res<Tunables>,
) {
    let tree_mesh = meshes.add(Cuboid::new(
        tunables.tree_size.x,
        tunables.tree_size.y,
        tunables.tree_size.z,
    ));
    let tree_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.6, 0.2),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });

    let rock_mesh = meshes.add(Cuboid::new(
        tunables.rock_size.x,
        tunables.rock_size.y,
        tunables.rock_size.z,
    ));
    let rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    commands.insert_resource(ChunkAssets {
        tree_mesh,
        tree_mat,
        rock_mesh,
        rock_mat,
    });
}

// Debug overlay is not implemented in this scaffold; enable logging instead.

pub fn world_to_chunk(pos: Vec3, size: f32) -> ChunkCoord {
    let fx = pos.x.div_euclid(size).floor();
    let fz = pos.z.div_euclid(size).floor();
    ChunkCoord {
        x: fx as i32,
        z: fz as i32,
    }
}

fn chunk_origin(coord: ChunkCoord, size: f32) -> Vec3 {
    Vec3::new(coord.x as f32 * size, 0.0, coord.z as f32 * size)
}

fn despawn_recursive(commands: &mut Commands, entity: Entity, children_q: &Query<&Children>) {
    if let Ok(children) = children_q.get(entity) {
        for i in 0..children.len() {
            let child = children[i];
            despawn_recursive(commands, child, children_q);
        }
    }
    commands.entity(entity).despawn();
}

fn track_player_chunk(
    cfg: Res<ChunkConfig>,
    mut pc: ResMut<PlayerChunk>,
    player_q: Query<&Transform, With<Player>>,
) {
    if let Ok(t) = player_q.single() {
        pc.0 = world_to_chunk(t.translation, cfg.size);
    }
}

fn update_chunks(
    mut commands: Commands,
    cfg: Res<ChunkConfig>,
    seed: Res<WorldSeed>,
    pc: Res<PlayerChunk>,
    mut loaded: ResMut<LoadedChunks>,
    assets: Res<ChunkAssets>,
    tunables: Res<Tunables>,
    children_q: Query<&Children>,
) {
    let center = pc.0;
    let desired = desired_chunks(center, cfg.active_radius);
    let keep = desired_chunks(center, cfg.active_radius + cfg.hysteresis);

    // Compute unload list (outside keep)
    let mut to_unload: Vec<ChunkCoord> = loaded
        .0
        .keys()
        .copied()
        .filter(|c| !keep.contains(c))
        .collect();
    to_unload.truncate(cfg.max_unloads_per_frame.min(to_unload.len()));

    // Prepare a query to fetch children for manual recursive despawn
    // Note: we cannot query here; this is a system param-only place. We will despawn root (children will remain)
    // To keep simple for now, ensure we spawn resources as direct children and rely on GC pass in future.
    for coord in to_unload {
        if let Some(entity) = loaded.0.remove(&coord) {
            despawn_recursive(&mut commands, entity, &children_q);
        }
    }

    // Compute load list (in desired but not loaded)
    let mut to_load: Vec<ChunkCoord> = desired
        .iter()
        .filter(|c| !loaded.0.contains_key(c))
        .copied()
        .collect();
    to_load.truncate(cfg.max_loads_per_frame.min(to_load.len()));

    for coord in to_load {
        let root = commands
            .spawn((
                Name::new(format!("Chunk ({}, {})", coord.x, coord.z)),
                ChunkRoot,
                Transform::IDENTITY,
                Visibility::default(),
            ))
            .id();

        spawn_chunk_content(
            root,
            coord,
            &mut commands,
            &assets,
            &tunables,
            seed.0,
            cfg.size,
        );

        loaded.0.insert(coord, root);
    }
}

fn chunk_hud_toggle(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut hud: ResMut<ChunkHudState>,
    children_q: Query<&Children>,
) {
    if input.just_pressed(KeyCode::F3) {
        hud.enabled = !hud.enabled;
        if !hud.enabled {
            if let Some(root) = hud.root.take() {
                despawn_recursive(&mut commands, root, &children_q);
            }
        }
    }

    if hud.enabled && hud.root.is_none() {
        let root = commands
            .spawn((
                ChunkHudRoot,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(12.0),
                    top: Val::Px(12.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|parent| {
                parent.spawn((
                    ChunkHudText,
                    Text::new("chunks: ..."),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            })
            .id();
        hud.root = Some(root);
    }
}

fn update_chunk_hud_text(
    hud: Res<ChunkHudState>,
    loaded: Res<LoadedChunks>,
    player_chunk: Res<PlayerChunk>,
    mut text_q: Query<&mut Text, With<ChunkHudText>>,
) {
    if !hud.enabled {
        return;
    }
    if let Ok(mut text) = text_q.single_mut() {
        *text = Text::new(format!(
            "chunk ({}, {}), loaded {}",
            player_chunk.0.x,
            player_chunk.0.z,
            loaded.0.len()
        ));
    }
}

fn chunk_config_shortcuts(input: Res<ButtonInput<KeyCode>>, mut cfg: ResMut<ChunkConfig>) {
    // Active radius: F8(-), F9(+)
    if input.just_pressed(KeyCode::F8) {
        cfg.active_radius = (cfg.active_radius - 1).max(1);
    }
    if input.just_pressed(KeyCode::F9) {
        cfg.active_radius = (cfg.active_radius + 1).min(6);
    }

    // Hysteresis: F10(-), F11(+)
    if input.just_pressed(KeyCode::F10) {
        cfg.hysteresis = (cfg.hysteresis - 1).max(0);
    }
    if input.just_pressed(KeyCode::F11) {
        cfg.hysteresis = (cfg.hysteresis + 1).min(3);
    }

    // Load caps: F6(-), F7(+)
    if input.just_pressed(KeyCode::F6) {
        cfg.max_loads_per_frame = (cfg.max_loads_per_frame.saturating_sub(1)).max(1);
    }
    if input.just_pressed(KeyCode::F7) {
        cfg.max_loads_per_frame = (cfg.max_loads_per_frame + 1).min(8);
    }

    // Unload caps: F4(-), F5(+)
    if input.just_pressed(KeyCode::F4) {
        cfg.max_unloads_per_frame = (cfg.max_unloads_per_frame.saturating_sub(1)).max(1);
    }
    if input.just_pressed(KeyCode::F5) {
        cfg.max_unloads_per_frame = (cfg.max_unloads_per_frame + 1).min(16);
    }
}

fn distance_culling(
    cfg: Res<ChunkConfig>,
    player_q: Query<&Transform, With<Player>>,
    mut q: Query<(&Transform, &mut Visibility), With<Mesh3d>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let threshold = cfg.size * (cfg.active_radius + cfg.hysteresis + 1) as f32;
    let p = player_tf.translation;
    for (tf, mut vis) in q.iter_mut() {
        let d = Vec2::new(tf.translation.x - p.x, tf.translation.z - p.z).length();
        *vis = if d > threshold {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

fn desired_chunks(center: ChunkCoord, r: i32) -> HashSet<ChunkCoord> {
    let mut set = HashSet::new();
    for dz in -r..=r {
        for dx in -r..=r {
            set.insert(ChunkCoord {
                x: center.x + dx,
                z: center.z + dz,
            });
        }
    }
    set
}

fn hash_combine(seed: u64, x: i32, z: i32) -> u64 {
    let mut h = seed ^ 0x9E37_79B9_7F4A_7C15u64;
    h ^= (x as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4Fu64);
    h = h.rotate_left(27) ^ (h >> 33);
    h ^= (z as u64).wrapping_mul(0x1656_67B1_9E37_79F9u64);
    h ^ (h >> 29)
}

fn spawn_chunk_content(
    root: Entity,
    coord: ChunkCoord,
    commands: &mut Commands,
    assets: &Res<ChunkAssets>,
    tunables: &Res<Tunables>,
    world_seed: u64,
    size: f32,
) {
    let origin = chunk_origin(coord, size);
    let mut rng = StdRng::seed_from_u64(hash_combine(world_seed, coord.x, coord.z));

    // Increase resource density using tunables as a guide
    let trees_per_chunk = (tunables.trees_count / 2).max(30) as usize;
    let rocks_per_chunk = (tunables.rocks_count / 2).max(12) as usize;

    // Trees
    for _ in 0..trees_per_chunk {
        let local_x = rng.random::<f32>() * size;
        let local_z = rng.random::<f32>() * size;
        let pos = origin + Vec3::new(local_x, 0.0, local_z);

        // Random wood amount per tree within tunables range
        let wood_span = (tunables.tree_wood_max - tunables.tree_wood_min + 1).max(1);
        let wood_amount = tunables.tree_wood_min + (rng.random::<u32>() % wood_span);

        commands.entity(root).with_children(|p| {
            p.spawn((
                Mesh3d(assets.tree_mesh.clone()),
                MeshMaterial3d(assets.tree_mat.clone()),
                Transform::from_xyz(pos.x, 1.5, pos.z),
                Tree,
                Harvestable {
                    kind: HarvestableKind::Wood,
                    amount: wood_amount,
                },
            ));
        });
    }

    // Rocks
    for _ in 0..rocks_per_chunk {
        let local_x = rng.random::<f32>() * size;
        let local_z = rng.random::<f32>() * size;
        let pos = origin + Vec3::new(local_x, 0.0, local_z);

        commands.entity(root).with_children(|p| {
            p.spawn((
                Mesh3d(assets.rock_mesh.clone()),
                MeshMaterial3d(assets.rock_mat.clone()),
                Transform::from_xyz(pos.x, 0.3, pos.z),
                Harvestable {
                    kind: HarvestableKind::Rock,
                    amount: 10,
                },
            ));
        });
    }
}
