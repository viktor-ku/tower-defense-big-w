use crate::{components::*, systems::CameraSettings};
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
) {
    // Insert global camera settings resource (easy to tweak)
    commands.insert_resource(CameraSettings {
        offset: Vec3::new(0.0, 80.0, 50.0), // Further away: height 80, distance 50
    });

    commands.spawn((
        Camera3d::default(),
        // Initial camera pose will be overridden by camera_system every frame based on settings
        Transform::from_xyz(0.0, 80.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let ground_mesh = meshes.add(Plane3d::default().mesh().size(200.0, 200.0).build());
    let ground_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.35, 0.2),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_mat),
        Transform::IDENTITY,
    ));

    // Roads: four curvy strips leading to village center, distinct dark material
    let road_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.15),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });

    let road_width = 6.0; // narrower roads

    // Generate random road patterns for each direction
    let mut all_roads = Vec::new();

    // Roads connect directly to village center (no town square)

    // North road (from z=-100 to village center)
    let north = generate_and_spawn_road(
        &mut commands,
        &mut meshes,
        road_mat.clone(),
        Vec3::new(0.0, 0.0, -100.0),
        Vec3::new(0.0, 0.0, 0.0), // Direct to village center
        road_width,
    )
    .unwrap();
    all_roads.push(north);

    // South road (from z=100 to village center)
    let south = generate_and_spawn_road(
        &mut commands,
        &mut meshes,
        road_mat.clone(),
        Vec3::new(0.0, 0.0, 100.0),
        Vec3::new(0.0, 0.0, 0.0), // Direct to village center
        road_width,
    )
    .unwrap();
    all_roads.push(south);

    // West road (from x=-100 to village center)
    let west = generate_and_spawn_road(
        &mut commands,
        &mut meshes,
        road_mat.clone(),
        Vec3::new(-100.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0), // Direct to village center
        road_width,
    )
    .unwrap();
    all_roads.push(west);

    // East road (from x=100 to village center)
    let east = generate_and_spawn_road(
        &mut commands,
        &mut meshes,
        road_mat.clone(),
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0), // Direct to village center
        road_width,
    )
    .unwrap();
    all_roads.push(east);

    // Save roads for path following
    commands.insert_resource(RoadPaths { roads: all_roads });

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

    // Spawn village (center) - Big purple block for visibility
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
            health: 100,
            max_health: 100,
        },
    ));

    // Spawn trees around the map
    for i in 0..15 {
        let angle = (i as f32 / 15.0) * 2.0 * std::f32::consts::PI;
        let distance = 60.0 + rand::random::<f32>() * 80.0;
        let x = angle.cos() * distance;
        let y = angle.sin() * distance;

        // Random wood amount per tree
        let wood_amount = 15 + rand::random::<u32>() % 20; // 15-35 wood per tree

        let tree_mesh = meshes.add(Cuboid::new(1.2, 3.0, 1.2));
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

    // Spawn some rock resources
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * 2.0 * std::f32::consts::PI;
        let distance = 50.0 + rand::random::<f32>() * 60.0;
        let x = angle.cos() * distance;
        let y = angle.sin() * distance;

        let rock_mesh = meshes.add(Cuboid::new(0.8, 0.6, 0.8));
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
