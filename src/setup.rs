use crate::{
    components::{Resource as GameResource, *},
    systems::CameraSettings,
};
use bevy::prelude::*;

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

    // Helper to spawn a curved road along Bezier points and collect waypoints
    let mut spawn_curved_road = |p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, width: f32| -> Vec<Vec3> {
        let steps = 24;
        let y = 0.011; // slightly above ground to avoid z-fighting
        let mut last = p0;
        let mut waypoints = Vec::with_capacity(steps + 1);
        waypoints.push(Vec3::new(p0.x, 0.0, p0.z));
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let it = 1.0 - t;
            // Cubic bezier on XZ (keep Y fixed)
            let x = it * it * it * p0.x
                + 3.0 * it * it * t * p1.x
                + 3.0 * it * t * t * p2.x
                + t * t * t * p3.x;
            let z = it * it * it * p0.z
                + 3.0 * it * it * t * p1.z
                + 3.0 * it * t * t * p2.z
                + t * t * t * p3.z;
            let current = Vec3::new(x, y, z);
            waypoints.push(Vec3::new(x, 0.0, z));
            let dir = (current - last);
            let seg_len = Vec2::new(dir.x, dir.z).length().max(0.001);
            let mid = (current + last) * 0.5;
            // Yaw to align local X with segment direction
            let yaw = dir.z.atan2(dir.x);
            let rotation = Quat::from_rotation_y(yaw);

            // Mesh sized along local X by segment length and local Z by width
            let seg_mesh = meshes.add(Plane3d::default().mesh().size(seg_len, width).build());
            commands.spawn((
                Mesh3d(seg_mesh),
                MeshMaterial3d(road_mat.clone()),
                Transform {
                    translation: Vec3::new(mid.x, y, mid.z),
                    rotation,
                    scale: Vec3::ONE,
                },
            ));

            last = current;
        }
        waypoints
    };

    let road_width = 6.0; // narrower roads

    // North road (from z=-100 to center) with slight S-curve
    let north = spawn_curved_road(
        Vec3::new(0.0, 0.0, -100.0),
        Vec3::new(-25.0, 0.0, -70.0),
        Vec3::new(20.0, 0.0, -30.0),
        Vec3::new(0.0, 0.0, 0.0),
        road_width,
    );
    // South road
    let south = spawn_curved_road(
        Vec3::new(0.0, 0.0, 100.0),
        Vec3::new(25.0, 0.0, 70.0),
        Vec3::new(-20.0, 0.0, 30.0),
        Vec3::new(0.0, 0.0, 0.0),
        road_width,
    );
    // West road
    let west = spawn_curved_road(
        Vec3::new(-100.0, 0.0, 0.0),
        Vec3::new(-70.0, 0.0, -25.0),
        Vec3::new(-30.0, 0.0, 20.0),
        Vec3::new(0.0, 0.0, 0.0),
        road_width,
    );
    // East road
    let east = spawn_curved_road(
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::new(70.0, 0.0, 25.0),
        Vec3::new(30.0, 0.0, -20.0),
        Vec3::new(0.0, 0.0, 0.0),
        road_width,
    );

    // Save roads for path following
    commands.insert_resource(RoadPaths {
        roads: vec![north, south, west, east],
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
            Tree {
                wood_amount,
                is_chopped: false,
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
            GameResource {
                resource_type: ResourceType::Rock,
                amount: 10,
            },
        ));
    }

    // Spawn day/night cycle
    commands.spawn(DayNight {
        is_day: true,
        time_until_switch: 30.0, // 30 seconds per day/night cycle
        day_duration: 30.0,
        night_duration: 20.0,
    });

    // Spawn building mode
    commands.spawn(BuildingMode { is_active: false });
}
