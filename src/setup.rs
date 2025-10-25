use crate::constants::Tunables;
use crate::{
    components::*,
    systems::{CameraSettings, EnemyHealthBarAssets},
};
use bevy::prelude::*;

/// Road pattern types used for procedural road generation.
#[derive(Debug, Clone, Copy)]
enum RoadPattern {
    Straight,
    Curved,
    Snake,
}

/// Generates and spawns a road mesh between two points; returns the path waypoints.
fn generate_and_spawn_road(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    start: Vec3,
    end: Vec3,
    width: f32,
) -> Option<Vec<Vec3>> {
    let mut waypoints = generate_road_pattern(start, end, width)?;

    // Enforce exact endpoints to guarantee clean connections to the town square
    if let Some(first) = waypoints.first_mut() {
        *first = Vec3::new(start.x, 0.0, start.z);
    }
    if let Some(last) = waypoints.last_mut() {
        *last = Vec3::new(end.x, 0.0, end.z);
    }

    // Spawn road segments
    let mut last = waypoints[0];
    for &current in waypoints.iter().skip(1) {
        let dir = current - last;
        let seg_len = dir.length();
        if seg_len > 0.001 {
            let mid = (last + current) / 2.0;
            let yaw = dir.z.atan2(dir.x);
            let rotation = Quat::from_rotation_y(yaw);

            let seg_mesh = meshes.add(Plane3d::default().mesh().size(seg_len, width).build());
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
        last = current;
    }

    Some(waypoints)
}

/// Generates a random road path (straight, curved, snake) between two points.
fn generate_road_pattern(start: Vec3, end: Vec3, _width: f32) -> Option<Vec<Vec3>> {
    let pattern = match rand::random::<u8>() % 3 {
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
            let wiggle_amplitude = 6.0 + rand::random::<f32>() * 8.0; // 6-14 units
            let wiggle_frequency = 2.0 + rand::random::<f32>() * 3.0; // 2-5 waves
            let phase_offset = rand::random::<f32>() * 2.0 * std::f32::consts::PI; // Random phase

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
                let noise_x = (rand::random::<f32>() - 0.5) * noise_amplitude;
                let noise_z = (rand::random::<f32>() - 0.5) * noise_amplitude;

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
            let curve_strength = 20.0 + rand::random::<f32>() * 40.0; // 20-60 units
            let mid1_offset = 0.2 + rand::random::<f32>() * 0.3; // 0.2-0.5
            let mid2_offset = 0.5 + rand::random::<f32>() * 0.3; // 0.5-0.8

            let mid1 = start
                + (end - start) * mid1_offset
                + Vec3::new(
                    (rand::random::<f32>() - 0.5) * curve_strength,
                    0.0,
                    (rand::random::<f32>() - 0.5) * curve_strength,
                );
            let mid2 = start
                + (end - start) * mid2_offset
                + Vec3::new(
                    (rand::random::<f32>() - 0.5) * curve_strength,
                    0.0,
                    (rand::random::<f32>() - 0.5) * curve_strength,
                );

            let segments = 15 + (rand::random::<u8>() % 11) as usize; // 15-25 segments
            generate_bezier_curve(start, mid1, mid2, end, segments)
        }
        RoadPattern::Snake => {
            // S-shaped road with random variations
            let mut waypoints = Vec::new();
            let steps = 25 + (rand::random::<u8>() % 16) as usize; // 25-40 steps

            // Random variations for snake pattern
            let snake_amplitude = 20.0 + rand::random::<f32>() * 25.0; // 20-45 units
            let snake_frequency = 1.5 + rand::random::<f32>() * 2.0; // 1.5-3.5 waves
            let phase_offset = rand::random::<f32>() * 2.0 * std::f32::consts::PI;

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
    ));

    // Perimeter walls (visual, thick, not walkable)
    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.55, 0.56),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });

    let half = tunables.town_size / 2.0;
    let h2 = tunables.wall_height / 2.0;

    // North wall (X-aligned at z = -half)
    let north_wall = meshes.add(Cuboid::new(
        tunables.town_size,
        tunables.wall_height,
        tunables.wall_thickness,
    ));
    commands.spawn((
        Mesh3d(north_wall),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(0.0, h2, -half),
        Wall,
    ));

    // South wall (X-aligned at z = +half)
    let south_wall = meshes.add(Cuboid::new(
        tunables.town_size,
        tunables.wall_height,
        tunables.wall_thickness,
    ));
    commands.spawn((
        Mesh3d(south_wall),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(0.0, h2, half),
        Wall,
    ));

    // West wall (Z-aligned at x = -half)
    let west_wall = meshes.add(Cuboid::new(
        tunables.wall_thickness,
        tunables.wall_height,
        tunables.town_size,
    ));
    commands.spawn((
        Mesh3d(west_wall),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(-half, h2, 0.0),
        Wall,
    ));

    // East wall with gate opening centered at z=0
    let segment_len = (tunables.town_size - tunables.gate_width) / 2.0;

    // Top (positive Z) segment
    let east_top = meshes.add(Cuboid::new(
        tunables.wall_thickness,
        tunables.wall_height,
        segment_len,
    ));
    commands.spawn((
        Mesh3d(east_top),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(half, h2, tunables.gate_width / 2.0 + segment_len / 2.0),
        Wall,
    ));

    // Bottom (negative Z) segment
    let east_bottom = meshes.add(Cuboid::new(
        tunables.wall_thickness,
        tunables.wall_height,
        segment_len,
    ));
    commands.spawn((
        Mesh3d(east_bottom),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(half, h2, -(tunables.gate_width / 2.0 + segment_len / 2.0)),
        Wall,
    ));

    // Town square pavement material
    let square_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.35, 0.38),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });

    // Roads: single strip from east gate to village center, distinct dark material
    let road_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.15),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });

    let road_width = tunables.road_width;
    let half = tunables.town_size / 2.0;

    // Single east road from the gate opening to village center
    let east_road = generate_and_spawn_road(
        &mut commands,
        &mut meshes,
        road_mat.clone(),
        Vec3::new(half, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        road_width,
    )
    .unwrap();

    // Save roads for path following
    commands.insert_resource(RoadPaths {
        roads: vec![east_road],
    });

    // 3D player box (larger and more visible)
    let player_mesh = meshes.add(Cuboid::new(2.0, 4.0, 2.0));
    let player_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.2, 0.2), // Bright red for visibility
        perceptual_roughness: 0.6,
        metallic: 0.0,
        ..default()
    });
    let _player = commands
        .spawn((
            Mesh3d(player_mesh),
            MeshMaterial3d(player_mat),
            Transform::from_xyz(0.0, 2.0, 0.0), // Higher up so it's more visible
            IsoPlayer,
            Player { wood: 0, rock: 0 },
        ))
        .id();

    // Town square pavement under the center
    let square_mesh = meshes.add(
        Plane3d::default()
            .mesh()
            .size(tunables.square_size, tunables.square_size)
            .build(),
    );
    commands.spawn((
        Mesh3d(square_mesh),
        MeshMaterial3d(square_mat),
        Transform::from_xyz(0.0, 0.012, 0.0),
        TownSquare,
    ));

    // Spawn village (center) - Big purple block for visibility; marks TownCenter
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
        Transform::from_xyz(0.0, 3.0, 0.0), // Elevated so it's visible
        Village {
            health: tunables.village_health,
            max_health: tunables.village_health,
        },
        TownCenter,
    ));

    // Spawn trees around the map (inside walls, avoid square and road corridor)
    let half = tunables.town_size / 2.0;
    for i in 0..tunables.trees_count {
        let angle = (i as f32 / 15.0) * 2.0 * std::f32::consts::PI;
        let distance = tunables.tree_distance_min
            + rand::random::<f32>() * (tunables.tree_distance_max - tunables.tree_distance_min);
        let x = angle.cos() * distance;
        let y = angle.sin() * distance; // z coordinate

        // Exclude town square vicinity
        let square_half = tunables.square_size / 2.0 + 8.0;
        if x.abs() <= square_half && y.abs() <= square_half {
            continue;
        }

        // Keep road corridor from east gate clear
        if x >= 0.0 && x <= half && y.abs() <= tunables.road_width * 1.8 {
            continue;
        }

        // Random wood amount per tree
        let wood_span = (tunables.tree_wood_max - tunables.tree_wood_min + 1).max(1);
        let wood_amount = tunables.tree_wood_min + rand::random::<u32>() % wood_span;

        let tree_mesh = meshes.add(Cuboid::new(
            tunables.tree_size.x,
            tunables.tree_size.y,
            tunables.tree_size.z,
        ));
        let tree_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 0.2), // Green for trees
            perceptual_roughness: 0.8,
            metallic: 0.0,
            ..default()
        });

        commands.spawn((
            Mesh3d(tree_mesh),
            MeshMaterial3d(tree_mat),
            Transform::from_xyz(x, 1.5, y),
            Tree,
            Harvestable {
                kind: HarvestableKind::Wood,
                amount: wood_amount,
            },
        ));
    }

    // Spawn some rock resources (inside walls, avoid square and road corridor)
    for i in 0..tunables.rocks_count {
        let angle = (i as f32 / 8.0) * 2.0 * std::f32::consts::PI;
        let distance = tunables.rock_distance_min
            + rand::random::<f32>() * (tunables.rock_distance_max - tunables.rock_distance_min);
        let x = angle.cos() * distance;
        let y = angle.sin() * distance; // z coordinate

        // Exclude town square vicinity
        let square_half = tunables.square_size / 2.0 + 8.0;
        if x.abs() <= square_half && y.abs() <= square_half {
            continue;
        }

        // Keep road corridor from east gate clear
        if x >= 0.0 && x <= half && y.abs() <= tunables.road_width * 1.8 {
            continue;
        }

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

        commands.spawn((
            Mesh3d(rock_mesh),
            MeshMaterial3d(rock_mat),
            Transform::from_xyz(x, 0.3, y),
            Harvestable {
                kind: HarvestableKind::Rock,
                amount: 10,
            },
        ));
    }

    // Spawn building mode
    commands.spawn(BuildingMode { is_active: false });
}
