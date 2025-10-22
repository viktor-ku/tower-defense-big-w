use crate::{
    components::{Resource as GameResource, *},
    systems::CameraFollow,
};
use bevy::prelude::*;

pub fn setup(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!(
        "Controls: WASD / Arrows to move. E to collect wood from trees. B to toggle build. ESC to menu/quit."
    );
    // Isometric 3D camera and light - positioned further back to see more of the world
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(25.0, 30.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraFollow {
            offset: Vec3::new(0.0, 20.0, 0.0), // Camera stays above and behind player
            follow_speed: 3.0,                 // How fast camera follows (higher = faster)
            edge_threshold: 0.15,              // 15% of screen edge triggers movement
        },
    ));
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // Ground plane
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

    // 3D player box (1x1x2 units approx)
    let player_mesh = meshes.add(Cuboid::new(1.0, 2.0, 1.0));
    let player_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.4, 0.9),
        perceptual_roughness: 0.6,
        metallic: 0.0,
        ..default()
    });
    let _player = commands
        .spawn((
            Mesh3d(player_mesh),
            MeshMaterial3d(player_mat),
            Transform::from_xyz(0.0, 1.0, 0.0),
            IsoPlayer,
            Player {
                speed: 200.0,
                wood: 0,
                rock: 0,
            },
        ))
        .id();

    // Spawn village (center)
    commands.spawn((
        Village {
            health: 100,
            max_health: 100,
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
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
                max_wood: wood_amount,
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
    commands.spawn(BuildingMode {
        is_active: false,
        tower_type: TowerType::Basic,
    });
}
