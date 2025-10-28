use crate::audio::AudioListener;
use crate::constants::Tunables;
use crate::random_policy::RandomizationPolicy;
use crate::{
    components::*,
    systems::{CameraSettings, EnemyHealthBarAssets},
};
use bevy::prelude::*;
use rand::{Rng, SeedableRng, rngs::StdRng};

/// Road pattern types used for procedural road generation.
#[derive(Debug, Clone, Copy)]
enum RoadPattern {
    Straight,
    Curved,
    Snake,
}

#[derive(Debug, Clone, Copy)]
enum ExitSide {
    North,
    East,
    South,
    West,
}

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
        let dir = current - last;
        let seg_len = dir.length();
        if seg_len > 0.001 {
            let yaw = dir.z.atan2(dir.x);
            let rotation = Quat::from_rotation_y(yaw);
            let forward = dir / seg_len; // normalized direction on XZ

            // Determine how many patches to render for this segment
            // Target small patch length for a tiled appearance
            let target_patch_len = 3.0_f32;
            let patch_count = (seg_len / target_patch_len).ceil().max(1.0) as u32;
            let patch_len = seg_len / patch_count as f32;

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
fn generate_road_pattern(
    start: Vec3,
    end: Vec3,
    _width: f32,
    rng: &mut StdRng,
) -> Option<Vec<Vec3>> {
    let pattern = match rng.random_range(0..3) {
        0 => RoadPattern::Straight,
        1 => RoadPattern::Curved,
        2 => RoadPattern::Snake,
        _ => RoadPattern::Straight,
    };

    match pattern {
        RoadPattern::Straight => {
            // Almost straight line with subtle wiggling and random variations
            let mut waypoints = Vec::new();
            let steps = 20;

            // Random variations for this road
            let wiggle_amplitude = 6.0 + rng.random::<f32>() * 8.0; // 6-14 units
            let wiggle_frequency = 2.0 + rng.random::<f32>() * 3.0; // 2-5 waves
            let phase_offset = rng.random::<f32>() * 2.0 * std::f32::consts::PI; // Random phase

            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let base_point = start.lerp(end, t);

                // Add subtle wiggling with random variations
                // Fade offsets near endpoints so connections are always clean
                let edge_fade = (t * (1.0 - t)).max(0.0).sqrt();
                let wiggle_offset = (t * std::f32::consts::PI * wiggle_frequency + phase_offset)
                    .sin()
                    * wiggle_amplitude
                    * edge_fade;

                // Add some random noise for extra variation
                let noise_amplitude = 3.0 * edge_fade;
                let noise_x = (rng.random::<f32>() - 0.5) * noise_amplitude;
                let noise_z = (rng.random::<f32>() - 0.5) * noise_amplitude;

                // Calculate perpendicular direction for wiggling
                let main_direction = (end - start).normalize();
                let perpendicular = Vec3::new(-main_direction.z, 0.0, main_direction.x);

                let wiggled_point =
                    base_point + perpendicular * wiggle_offset + Vec3::new(noise_x, 0.0, noise_z);
                waypoints.push(wiggled_point);
            }
            Some(waypoints)
        }
        RoadPattern::Curved => {
            // Curved road with random control points and variations
            let curve_strength = 20.0 + rng.random::<f32>() * 40.0; // 20-60 units
            let mid1_offset = 0.2 + rng.random::<f32>() * 0.3; // 0.2-0.5
            let mid2_offset = 0.5 + rng.random::<f32>() * 0.3; // 0.5-0.8

            let mid1 = start
                + (end - start) * mid1_offset
                + Vec3::new(
                    (rng.random::<f32>() - 0.5) * curve_strength,
                    0.0,
                    (rng.random::<f32>() - 0.5) * curve_strength,
                );
            let mid2 = start
                + (end - start) * mid2_offset
                + Vec3::new(
                    (rng.random::<f32>() - 0.5) * curve_strength,
                    0.0,
                    (rng.random::<f32>() - 0.5) * curve_strength,
                );

            let segments = 15 + (rng.random::<u8>() % 11) as usize; // 15-25 segments
            generate_bezier_curve(start, mid1, mid2, end, segments)
        }
        RoadPattern::Snake => {
            // S-shaped road with random variations
            let mut waypoints = Vec::new();
            let steps = 25 + (rng.random::<u8>() % 16) as usize; // 25-40 steps

            // Random variations for snake pattern
            let snake_amplitude = 20.0 + rng.random::<f32>() * 25.0; // 20-45 units
            let snake_frequency = 1.5 + rng.random::<f32>() * 2.0; // 1.5-3.5 waves
            let phase_offset = rng.random::<f32>() * 2.0 * std::f32::consts::PI;

            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let base_point = start.lerp(end, t);

                // Add S-curve offset with random variations
                // Fade offsets near endpoints so connections are always clean
                let edge_fade = (t * (1.0 - t)).max(0.0).sqrt();
                let offset = (t * std::f32::consts::PI * snake_frequency + phase_offset).sin()
                    * snake_amplitude
                    * edge_fade;

                // Add secondary wave for more complex snake pattern
                let secondary_amplitude = snake_amplitude * 0.3;
                let secondary_frequency = snake_frequency * 2.0;
                let secondary_offset = (t * std::f32::consts::PI * secondary_frequency).cos()
                    * (secondary_amplitude * edge_fade);

                let perpendicular = Vec3::new(-(end.z - start.z), 0.0, end.x - start.x).normalize();
                let point = base_point + perpendicular * (offset + secondary_offset);
                waypoints.push(point);
            }
            Some(waypoints)
        }
    }
}

/// Generates points on a cubic Bezier curve.
fn generate_bezier_curve(
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    num_segments: usize,
) -> Option<Vec<Vec3>> {
    let mut points = Vec::new();
    for i in 0..=num_segments {
        let t = i as f32 / num_segments as f32;
        let omt = 1.0 - t;
        let point = omt.powf(3.0) * p0
            + 3.0 * omt.powf(2.0) * t * p1
            + 3.0 * omt * t.powf(2.0) * p2
            + t.powf(3.0) * p3;
        points.push(point);
    }
    Some(points)
}

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

    // Choose exit side and gate lateral offset
    let exit_side = match rng.random_range(0..4) {
        0 => ExitSide::North,
        1 => ExitSide::East,
        2 => ExitSide::South,
        _ => ExitSide::West,
    };
    let m = tunables
        .gate_corner_margin
        .min(half - tunables.gate_width * 0.5 - 0.1);
    let lateral = rng.random_range((-half + m)..=(half - m));

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
            Player { wood: 0, rock: 0 },
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
