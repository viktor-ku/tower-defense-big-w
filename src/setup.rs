use bevy::prelude::*;
use crate::components::{Resource as GameResource, *};

pub fn setup(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!(
        "Controls: WASD / Arrows to move. B to toggle build. ESC to menu/quit."
    );
    // Isometric 3D camera and light
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(12.0, 14.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight { shadows_enabled: true, illuminance: 10000.0, ..default() },
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
    commands.spawn((Mesh3d(ground_mesh), MeshMaterial3d(ground_mat), Transform::IDENTITY));
    
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
            Player { speed: 200.0 },
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
    
    // Spawn some resources
    for i in 0..10 {
        let angle = (i as f32 / 10.0) * 2.0 * std::f32::consts::PI;
        let distance = 80.0 + rand::random::<f32>() * 40.0;
        let x = angle.cos() * distance;
        let y = angle.sin() * distance;
        
        let resource_type = if rand::random::<bool>() {
            ResourceType::Wood
        } else {
            ResourceType::Rock
        };
        
        let color = match resource_type {
            ResourceType::Wood => Color::srgb(0.6, 0.4, 0.2),
            ResourceType::Rock => Color::srgb(0.5, 0.5, 0.5),
        };
        
        let res_mesh = meshes.add(Cuboid::new(0.8, 0.6, 0.8));
        let res_mat = materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..default()
        });
        let _res = commands
            .spawn((
                Mesh3d(res_mesh),
                MeshMaterial3d(res_mat),
                Transform::from_xyz(x, 0.3, y),
                GameResource { resource_type, amount: 10 },
            ))
            .id();
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
