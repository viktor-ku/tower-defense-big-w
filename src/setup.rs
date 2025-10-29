use crate::audio::AudioListener;
use crate::components::*;
use crate::constants::Tunables;
use crate::core::paths::{generate_road_pattern, segment_patch_tiling};
use crate::core::world::{ExitSide, choose_exit_side, gate_lateral_offset};
use crate::random_policy::RandomizationPolicy;
use crate::systems::camera::CameraSettings;
use crate::systems::combat::assets::EnemyHealthBarAssets;
use bevy::prelude::*;
use rand::{Rng, SeedableRng, rngs::StdRng};

// ExitSide, choose_exit_side, gate_lateral_offset moved to core::world

/// Generates and spawns a road mesh between two points; returns the path waypoints.
fn generate_and_spawn_road(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    start: Vec3,
    end: Vec3,
    width: f32,
    rng: &mut StdRng,
) -> Option<Vec<Vec3>> {
    let mut waypoints = generate_road_pattern(start, end, width, rng)?;

    // Enforce exact endpoints to guarantee clean connections to the town square
    if let Some(first) = waypoints.first_mut() {
        *first = Vec3::new(start.x, 0.0, start.z);
    }
    if let Some(last) = waypoints.last_mut() {
        *last = Vec3::new(end.x, 0.0, end.z);
    }

    // Spawn road segments as multiple short patches for a tiled look (cosmetic only)
    let mut last = waypoints[0];
    for &current in waypoints.iter().skip(1) {
        if let Some((patch_count, patch_len, forward, yaw)) =
            segment_patch_tiling(last, current, 3.0)
        {
            let rotation = Quat::from_rotation_y(yaw);
            for i in 0..patch_count {
                let center_offset = (i as f32 + 0.5) * patch_len;
                let mid = last + forward * center_offset;
                let seg_mesh = meshes.add(Plane3d::default().mesh().size(patch_len, width).build());
                commands.spawn((
                    Mesh3d(seg_mesh),
                    MeshMaterial3d(material.clone()),
                    Transform {
                        translation: Vec3::new(mid.x, 0.011, mid.z),
                        rotation,
                        scale: Vec3::ONE,
                    },
                ));
            }
        }
        last = current;
    }

    Some(waypoints)
}

/// Generates a random road path (straight, curved, snake) between two points.
// generate_road_pattern moved to core::paths

/// Sets up the world: camera, light, ground, roads, player, village, trees, rocks, and systems state.
pub fn setup(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tunables: Res<Tunables>,
    policy: Res<RandomizationPolicy>,
) {
    // Insert global camera settings resource (easy to tweak)
    commands.insert_resource(CameraSettings {
        offset: tunables.camera_offset,
    });
    commands.insert_resource(EnemyHealthBarAssets::default());

    commands.spawn((
        Camera3d::default(),
        // Initial camera pose will be overridden by camera_system every frame based on settings
        Transform::from_xyz(
            tunables.camera_offset.x,
            tunables.camera_offset.y,
            tunables.camera_offset.z,
        )
        .looking_at(Vec3::ZERO, Vec3::Y),
        AudioListener,
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: tunables.light_illuminance,
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let ground_mesh = meshes.add(
        Plane3d::default()
            .mesh()
            .size(tunables.ground_size, tunables.ground_size)
            .build(),
    );
    let ground_mat = materials.add(StandardMaterial {
        base_color: tunables.ground_color,
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_mat),
        Transform::IDENTITY,
        NoDistanceCull,
    ));

    // Perimeter walls and seeded exit gate
    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.55, 0.56),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });

    let half = tunables.town_size / 2.0;
    let h2 = tunables.wall_height / 2.0;

    // RNG for layout (seeded vs random per policy)
    let mut rng = if policy.town_layout_seeded {
        StdRng::seed_from_u64(tunables.world_seed)
    } else {
        let s: u64 = rand::rng().random();
        StdRng::seed_from_u64(s)
    };

    // Choose exit side and gate lateral offset (pure helpers)
    let exit_side = choose_exit_side(&mut rng);
    let lateral = gate_lateral_offset(
        &mut rng,
        half,
        tunables.gate_width,
        tunables.gate_corner_margin,
    );

    // Spawn walls with a gate opening on the chosen exit side
    let gate_center = match exit_side {
        ExitSide::East => {
            // Split east wall into two segments along Z
            let top_len = (half - (lateral + tunables.gate_width * 0.5)).max(0.0);
            if top_len > 0.0 {
                let mesh = meshes.add(Cuboid::new(
                    tunables.wall_thickness,
                    tunables.wall_height,
                    top_len,
                ));
                let z = lateral + tunables.gate_width * 0.5 + top_len * 0.5;
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(half, h2, z),
                    Wall,
                ));
            }
            let bottom_len = (lateral - tunables.gate_width * 0.5 - (-half)).max(0.0);
            if bottom_len > 0.0 {
                let mesh = meshes.add(Cuboid::new(
                    tunables.wall_thickness,
                    tunables.wall_height,
                    bottom_len,
                ));
                let z = -half + bottom_len * 0.5;
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(half, h2, z),
                    Wall,
                ));
            }
            // Other full walls
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.town_size,
                    tunables.wall_height,
                    tunables.wall_thickness,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(0.0, h2, -half),
                    Wall,
                ));
            }
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.town_size,
                    tunables.wall_height,
                    tunables.wall_thickness,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(0.0, h2, half),
                    Wall,
                ));
            }
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.wall_thickness,
                    tunables.wall_height,
                    tunables.town_size,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(-half, h2, 0.0),
                    Wall,
                ));
            }
            Vec3::new(half, 0.0, lateral)
        }
        ExitSide::West => {
            // Split west wall into two segments along Z
            let top_len = (half - (lateral + tunables.gate_width * 0.5)).max(0.0);
            if top_len > 0.0 {
                let mesh = meshes.add(Cuboid::new(
                    tunables.wall_thickness,
                    tunables.wall_height,
                    top_len,
                ));
                let z = lateral + tunables.gate_width * 0.5 + top_len * 0.5;
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(-half, h2, z),
                    Wall,
                ));
            }
            let bottom_len = (lateral - tunables.gate_width * 0.5 - (-half)).max(0.0);
            if bottom_len > 0.0 {
                let mesh = meshes.add(Cuboid::new(
                    tunables.wall_thickness,
                    tunables.wall_height,
                    bottom_len,
                ));
                let z = -half + bottom_len * 0.5;
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(-half, h2, z),
                    Wall,
                ));
            }
            // Other full walls
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.town_size,
                    tunables.wall_height,
                    tunables.wall_thickness,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(0.0, h2, -half),
                    Wall,
                ));
            }
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.town_size,
                    tunables.wall_height,
                    tunables.wall_thickness,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(0.0, h2, half),
                    Wall,
                ));
            }
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.wall_thickness,
                    tunables.wall_height,
                    tunables.town_size,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(half, h2, 0.0),
                    Wall,
                ));
            }
            Vec3::new(-half, 0.0, lateral)
        }
        ExitSide::North => {
            // Split north wall into two segments along X
            let right_len = (half - (lateral + tunables.gate_width * 0.5)).max(0.0);
            if right_len > 0.0 {
                let mesh = meshes.add(Cuboid::new(
                    right_len,
                    tunables.wall_height,
                    tunables.wall_thickness,
                ));
                let x = lateral + tunables.gate_width * 0.5 + right_len * 0.5;
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(x, h2, -half),
                    Wall,
                ));
            }
            let left_len = (lateral - tunables.gate_width * 0.5 - (-half)).max(0.0);
            if left_len > 0.0 {
                let mesh = meshes.add(Cuboid::new(
                    left_len,
                    tunables.wall_height,
                    tunables.wall_thickness,
                ));
                let x = -half + left_len * 0.5;
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(x, h2, -half),
                    Wall,
                ));
            }
            // Other full walls
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.town_size,
                    tunables.wall_height,
                    tunables.wall_thickness,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(0.0, h2, half),
                    Wall,
                ));
            }
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.wall_thickness,
                    tunables.wall_height,
                    tunables.town_size,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(-half, h2, 0.0),
                    Wall,
                ));
            }
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.wall_thickness,
                    tunables.wall_height,
                    tunables.town_size,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(half, h2, 0.0),
                    Wall,
                ));
            }
            Vec3::new(lateral, 0.0, -half)
        }
        ExitSide::South => {
            // Split south wall into two segments along X
            let right_len = (half - (lateral + tunables.gate_width * 0.5)).max(0.0);
            if right_len > 0.0 {
                let mesh = meshes.add(Cuboid::new(
                    right_len,
                    tunables.wall_height,
                    tunables.wall_thickness,
                ));
                let x = lateral + tunables.gate_width * 0.5 + right_len * 0.5;
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(x, h2, half),
                    Wall,
                ));
            }
            let left_len = (lateral - tunables.gate_width * 0.5 - (-half)).max(0.0);
            if left_len > 0.0 {
                let mesh = meshes.add(Cuboid::new(
                    left_len,
                    tunables.wall_height,
                    tunables.wall_thickness,
                ));
                let x = -half + left_len * 0.5;
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(x, h2, half),
                    Wall,
                ));
            }
            // Other full walls
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.town_size,
                    tunables.wall_height,
                    tunables.wall_thickness,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(0.0, h2, -half),
                    Wall,
                ));
            }
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.wall_thickness,
                    tunables.wall_height,
                    tunables.town_size,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(-half, h2, 0.0),
                    Wall,
                ));
            }
            {
                let mesh = meshes.add(Cuboid::new(
                    tunables.wall_thickness,
                    tunables.wall_height,
                    tunables.town_size,
                ));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(wall_mat.clone()),
                    Transform::from_xyz(half, h2, 0.0),
                    Wall,
                ));
            }
            Vec3::new(lateral, 0.0, half)
        }
    };

    // Town square pavement material
    let square_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.35, 0.38),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });

    // Roads material
    let road_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.15),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });

    let road_width = tunables.road_width;

    // Determine base position near the wall opposite to the exit side
    let side_normal = match exit_side {
        ExitSide::East => Vec3::new(1.0, 0.0, 0.0),
        ExitSide::West => Vec3::new(-1.0, 0.0, 0.0),
        ExitSide::North => Vec3::new(0.0, 0.0, -1.0),
        ExitSide::South => Vec3::new(0.0, 0.0, 1.0),
    };
    let opposite_dir = -side_normal;
    let base_pos = opposite_dir * (half - tunables.base_clearance_from_wall);

    // (Player will be spawned on the TownSquare after it's placed)

    // Plaza (TownSquare): 2:1 wide rectangle in front of base, facing the gate
    let short_side = tunables.plaza_short_side;
    let long_side = tunables.plaza_aspect * short_side;
    let mut dir_to_gate = gate_center - base_pos;
    dir_to_gate.y = 0.0;
    let dir_len = dir_to_gate.length();
    let dir_to_gate = if dir_len > 1e-3 {
        dir_to_gate / dir_len
    } else {
        side_normal
    };
    let plaza_center = base_pos + dir_to_gate * (tunables.plaza_gap_from_base + 0.5 * short_side);
    let yaw = dir_to_gate.z.atan2(dir_to_gate.x);
    let plaza_rotation = Quat::from_rotation_y(yaw + std::f32::consts::FRAC_PI_2);
    let square_mesh = meshes.add(
        Plane3d::default()
            .mesh()
            .size(long_side, short_side)
            .build(),
    );
    commands.spawn((
        Mesh3d(square_mesh),
        MeshMaterial3d(square_mat),
        Transform {
            translation: Vec3::new(plaza_center.x, 0.012, plaza_center.z),
            rotation: plaza_rotation,
            scale: Vec3::ONE,
        },
        TownSquare,
    ));

    // Publish the plaza center so other systems (e.g., resource spawning) can respect it
    commands.insert_resource(TownSquareCenter(plaza_center));

    // 3D player box (larger and more visible) — spawn on the TownSquare
    let player_mesh = meshes.add(Cuboid::new(2.0, 4.0, 2.0));
    let player_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.2, 0.2),
        perceptual_roughness: 0.6,
        metallic: 0.0,
        ..default()
    });
    let _player = commands
        .spawn((
            Mesh3d(player_mesh),
            MeshMaterial3d(player_mat),
            Transform::from_xyz(plaza_center.x, 2.0, plaza_center.z),
            IsoPlayer,
            Player {
                wood: 0,
                rock: 0,
                silver: 0,
                gold: 0,
            },
        ))
        .id();

    // Spawn village (base) near opposite wall
    let village_mesh = meshes.add(Cuboid::new(8.0, 6.0, 8.0)); // Big block
    let village_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.8), // Bright purple
        perceptual_roughness: 0.7,
        metallic: 0.0,
        ..default()
    });

    commands.spawn((
        Mesh3d(village_mesh),
        MeshMaterial3d(village_mat),
        Transform::from_xyz(base_pos.x, 3.0, base_pos.z), // Elevated so it's visible
        Village {
            health: tunables.village_health,
            max_health: tunables.village_health,
        },
        TownCenter,
    ));

    // Road from gate to base using generated patterns (seeded vs random per policy)
    let road_seed = tunables.world_seed ^ 0xD00Du64.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut road_rng = if policy.road_generation_seeded {
        StdRng::seed_from_u64(road_seed)
    } else {
        let s: u64 = rand::rng().random();
        StdRng::seed_from_u64(s)
    };
    if let Some(road) = generate_and_spawn_road(
        &mut commands,
        &mut meshes,
        road_mat.clone(),
        gate_center,
        plaza_center,
        road_width,
        &mut road_rng,
    ) {
        commands.insert_resource(RoadPaths { roads: vec![road] });
    }

    // Trees and rocks are now spawned by the chunking system per active chunk

    // Spawn building and selling modes
    commands.spawn(BuildingMode { is_active: false });
    commands.spawn(SellingMode { is_active: false });
}
