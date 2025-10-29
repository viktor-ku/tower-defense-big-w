use crate::components::{
    ChunkRoot, Harvestable, HarvestableKind, NoDistanceCull, Player, Tree, TreeSize,
};
use crate::constants::Tunables;
use bevy::prelude::*;
// UI debug overlay omitted for now; logging is used instead
use crate::random_policy::RandomizationPolicy;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
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
    pub big_tree_mesh: Handle<Mesh>,
    pub big_tree_mat: Handle<StandardMaterial>,
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
            .add_systems(Startup, setup_chunk_assets)
            .add_systems(Startup, load_initial_chunks.after(setup_chunk_assets))
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

    // Big tree is 1.7x wider and 1.8x taller
    let big_tree_mesh = meshes.add(Cuboid::new(
        tunables.tree_size.x * 1.7,
        tunables.tree_size.y * 1.8,
        tunables.tree_size.z * 1.7,
    ));
    let big_tree_mat = materials.add(StandardMaterial {
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
        big_tree_mesh,
        big_tree_mat,
        rock_mesh,
        rock_mat,
    });
}

/// Load the initial chunk (0,0) and its adjacent chunks at game start.
fn load_initial_chunks(
    mut commands: Commands,
    cfg: Res<ChunkConfig>,
    seed: Res<WorldSeed>,
    mut loaded: ResMut<LoadedChunks>,
    assets: Res<ChunkAssets>,
    tunables: Res<Tunables>,
    policy: Res<RandomizationPolicy>,
) {
    let initial_coord = ChunkCoord { x: 0, z: 0 };

    // Load the initial chunk and its adjacent chunks
    let mut chunks_to_load = HashSet::new();
    chunks_to_load.insert(initial_coord);
    chunks_to_load.extend(adjacent_chunks(initial_coord));

    for coord in chunks_to_load {
        // Only load if not already loaded
        if !loaded.0.contains_key(&coord) {
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
                policy.chunk_content_seeded,
            );

            loaded.0.insert(coord, root);
        }
    }
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

#[allow(clippy::too_many_arguments)]
fn update_chunks(
    mut commands: Commands,
    cfg: Res<ChunkConfig>,
    seed: Res<WorldSeed>,
    pc: Res<PlayerChunk>,
    mut loaded: ResMut<LoadedChunks>,
    assets: Res<ChunkAssets>,
    tunables: Res<Tunables>,
    children_q: Query<&Children>,
    mut last_chunk: Local<Option<ChunkCoord>>,
    policy: Res<RandomizationPolicy>,
) {
    // Only perform load/unload work when the player actually changes chunks
    if *last_chunk == Some(pc.0) {
        return;
    }
    *last_chunk = Some(pc.0);
    let center = pc.0;
    let desired = desired_chunks(center, cfg.active_radius);
    let keep = desired_chunks(center, cfg.active_radius + cfg.hysteresis);

    // Also preload adjacent chunks for smoother resource availability
    let adjacent = adjacent_chunks(center);
    let mut all_desired = desired;
    all_desired.extend(adjacent);

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

    // Compute load list (in desired + adjacent but not loaded)
    let mut to_load: Vec<ChunkCoord> = all_desired
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
            policy.chunk_content_seeded,
        );

        loaded.0.insert(coord, root);
    }
}

fn chunk_hud_toggle(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut hud: ResMut<ChunkHudState>,
    children_q: Query<&Children>,
    asset_server: Res<AssetServer>,
) {
    if input.just_pressed(KeyCode::F3) {
        hud.enabled = !hud.enabled;
        if !hud.enabled
            && let Some(root) = hud.root.take()
        {
            despawn_recursive(&mut commands, root, &children_q);
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
                        font: asset_server.load("fonts/Nova_Mono/NovaMono-Regular.ttf"),
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
    if !hud.is_changed() && !loaded.is_changed() && !player_chunk.is_changed() {
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

#[derive(Default)]
struct DistanceCullState {
    last_player_pos: Vec3,
    initialized: bool,
    accumulator_secs: f32,
}

fn distance_culling(
    cfg: Res<ChunkConfig>,
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut q: Query<(&Transform, &mut Visibility), (With<Mesh3d>, Without<NoDistanceCull>)>,
    mut state: Local<DistanceCullState>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };

    // Throttle updates to ~10 Hz and only when the player has moved noticeably
    state.accumulator_secs += time.delta_secs();
    let moved_enough = if state.initialized {
        player_tf
            .translation
            .distance_squared(state.last_player_pos)
            > 1.0 // ~1 unit squared
    } else {
        true
    };

    if state.accumulator_secs < 0.1 && !moved_enough {
        return;
    }
    state.accumulator_secs = 0.0;
    state.last_player_pos = player_tf.translation;
    state.initialized = true;

    // Be less aggressive: add a safety margin to cover diagonal distances and large meshes
    let base = (cfg.active_radius + cfg.hysteresis + 1) as f32;
    let threshold = cfg.size * base * 1.5;
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

/// Get the 8 adjacent chunks (cardinal + diagonal) around a center chunk.
fn adjacent_chunks(center: ChunkCoord) -> HashSet<ChunkCoord> {
    let mut set = HashSet::new();

    // Cardinal directions
    // Up (north)
    set.insert(ChunkCoord {
        x: center.x,
        z: center.z + 1,
    });
    // Down (south)
    set.insert(ChunkCoord {
        x: center.x,
        z: center.z - 1,
    });
    // Left (west)
    set.insert(ChunkCoord {
        x: center.x - 1,
        z: center.z,
    });
    // Right (east)
    set.insert(ChunkCoord {
        x: center.x + 1,
        z: center.z,
    });

    // Diagonal directions
    // Up-Left (northwest)
    set.insert(ChunkCoord {
        x: center.x - 1,
        z: center.z + 1,
    });
    // Up-Right (northeast)
    set.insert(ChunkCoord {
        x: center.x + 1,
        z: center.z + 1,
    });
    // Down-Left (southwest)
    set.insert(ChunkCoord {
        x: center.x - 1,
        z: center.z - 1,
    });
    // Down-Right (southeast)
    set.insert(ChunkCoord {
        x: center.x + 1,
        z: center.z - 1,
    });

    set
}

fn hash_combine(seed: u64, x: i32, z: i32) -> u64 {
    let mut h = seed ^ 0x9E37_79B9_7F4A_7C15u64;
    h ^= (x as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4Fu64);
    h = h.rotate_left(27) ^ (h >> 33);
    h ^= (z as u64).wrapping_mul(0x1656_67B1_9E37_79F9u64);
    h ^ (h >> 29)
}

/// Generate a deterministic resource count for a chunk based on the world seed and chunk coordinates.
/// Returns a value between 250 and 275 (inclusive) that is reproducible for the same seed and chunk.
fn generate_chunk_resource_count(world_seed: u64, chunk_x: i32, chunk_z: i32) -> u32 {
    // Create a unique seed for this chunk's resource count
    let resource_seed = hash_combine(world_seed ^ 0x123456789ABCDEF0, chunk_x, chunk_z);
    let mut rng = StdRng::seed_from_u64(resource_seed);

    // Generate a value between 250 and 275 (inclusive)
    let range = 275 - 250 + 1; // 26 possible values
    250 + (rng.random::<u32>() % range)
}

fn spawn_chunk_content(
    root: Entity,
    coord: ChunkCoord,
    commands: &mut Commands,
    assets: &Res<ChunkAssets>,
    tunables: &Res<Tunables>,
    world_seed: u64,
    size: f32,
    seeded: bool,
) {
    let origin = chunk_origin(coord, size);
    let mut seeded_rng = StdRng::seed_from_u64(hash_combine(world_seed, coord.x, coord.z));
    let mut thread_rng = rand::rng();
    fn pick_f32(
        seeded: bool,
        seeded_rng: &mut StdRng,
        thread_rng: &mut rand::rngs::ThreadRng,
    ) -> f32 {
        if seeded {
            seeded_rng.random::<f32>()
        } else {
            thread_rng.random::<f32>()
        }
    }
    fn pick_u32(
        seeded: bool,
        seeded_rng: &mut StdRng,
        thread_rng: &mut rand::rngs::ThreadRng,
    ) -> u32 {
        if seeded {
            seeded_rng.random::<u32>()
        } else {
            thread_rng.random::<u32>()
        }
    }

    // Generate seed-based resource counts (200-250 total resources per chunk)
    let resource_count = generate_chunk_resource_count(world_seed, coord.x, coord.z);
    let trees_per_chunk = (resource_count * 2 / 3) as usize; // 2/3 trees, 1/3 rocks
    let rocks_per_chunk = (resource_count / 3) as usize;

    // Trees
    for _ in 0..trees_per_chunk {
        let local_x = pick_f32(seeded, &mut seeded_rng, &mut thread_rng) * size;
        let local_z = pick_f32(seeded, &mut seeded_rng, &mut thread_rng) * size;
        let pos = origin + Vec3::new(local_x, 0.0, local_z);

        // Skip if within town resource exclusion radius
        if pos.length() <= tunables.town_resource_exclusion_radius {
            continue;
        }

        // Determine if this is a big tree based on distance from village
        // Big trees are rare near village (5% chance) but common far away (50% chance)
        let distance_from_village = pos.length();
        let big_tree_chance = if distance_from_village < 100.0 {
            // Very close to village: 5% chance
            0.05
        } else if distance_from_village < 200.0 {
            // Medium distance: 20% chance
            0.20
        } else {
            // Far from village: 50% chance
            0.50
        };

        let is_big_tree = pick_f32(seeded, &mut seeded_rng, &mut thread_rng) < big_tree_chance;
        let tree_size = if is_big_tree {
            TreeSize::Big
        } else {
            TreeSize::Small
        };

        // Random wood amount per tree within tunables range
        let wood_span = (tunables.tree_wood_max - tunables.tree_wood_min + 1).max(1);
        let wood_amount = tunables.tree_wood_min
            + (pick_u32(seeded, &mut seeded_rng, &mut thread_rng) % wood_span);

        commands.entity(root).with_children(|p| {
            let (mesh, material) = if is_big_tree {
                (assets.big_tree_mesh.clone(), assets.big_tree_mat.clone())
            } else {
                (assets.tree_mesh.clone(), assets.tree_mat.clone())
            };

            p.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(pos.x, 1.5, pos.z),
                Tree,
                tree_size,
                Harvestable {
                    kind: HarvestableKind::Wood,
                    amount: wood_amount,
                },
            ));
        });
    }

    // Rocks
    for _ in 0..rocks_per_chunk {
        let local_x = pick_f32(seeded, &mut seeded_rng, &mut thread_rng) * size;
        let local_z = pick_f32(seeded, &mut seeded_rng, &mut thread_rng) * size;
        let pos = origin + Vec3::new(local_x, 0.0, local_z);

        // Skip if within town resource exclusion radius
        if pos.length() <= tunables.town_resource_exclusion_radius {
            continue;
        }

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
