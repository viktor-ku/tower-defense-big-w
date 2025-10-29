use bevy::prelude::*;

use td::components::enemies::{Enemy, EnemyHealthBarRoot};
use td::constants::Tunables;
use td::systems::combat::enemy::{
    cleanup_enemy_health_bars, face_enemy_health_bars, position_enemy_health_bars,
};

#[test]
fn cleanup_removes_bar_when_enemy_despawns() {
    let mut world = World::new();
    let enemy = world
        .spawn(Enemy {
            health: 1,
            max_health: 1,
            speed: 1.0,
            damage: 1,
        })
        .id();
    let bar = world
        .spawn((
            EnemyHealthBarRoot { owner: enemy },
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(cleanup_enemy_health_bars);

    // Despawn enemy and run cleanup
    world.despawn(enemy);
    schedule.run(&mut world);

    assert!(world.get_entity(bar).is_err());
}

#[test]
fn position_health_bar_tracks_owner_height_with_offset() {
    let mut world = World::new();
    world.insert_resource(Tunables::default());
    let enemy = world
        .spawn((GlobalTransform::from_xyz(3.0, 5.0, -2.0),))
        .id();
    let bar = world
        .spawn((EnemyHealthBarRoot { owner: enemy }, Transform::default()))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(position_enemy_health_bars);
    schedule.run(&mut world);

    let t = world.get::<Transform>(bar).unwrap();
    assert!((t.translation.x - 3.0).abs() < 1e-5);
    assert!((t.translation.z + 2.0).abs() < 1e-5);
}

#[test]
fn face_health_bar_yaws_toward_camera() {
    let mut world = World::new();
    // Camera rotated 90 degrees yaw
    world.spawn((
        Camera3d::default(),
        GlobalTransform::from(Transform::from_rotation(Quat::from_rotation_y(
            std::f32::consts::FRAC_PI_2,
        ))),
    ));
    let owner = world.spawn_empty().id();
    let bar = world
        .spawn((
            EnemyHealthBarRoot { owner },
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(face_enemy_health_bars);
    schedule.run(&mut world);

    let t = world.get::<Transform>(bar).unwrap();
    // Rotated Z axis should align with -X for +90deg yaw (right-handed, -Z forward)
    let v = (t.rotation * Vec3::Z).normalize_or_zero();
    assert!((v.x + 1.0).abs() < 1e-3 && v.z.abs() < 1e-3);
}
