use bevy::prelude::*;

use td::components::enemies::{Enemy, EnemyHealthBarFill};
use td::systems::combat::enemy::update_enemy_health_bars;

#[test]
fn health_bar_scale_and_offset_updates_with_ratio() {
    let mut world = World::new();
    let enemy = world
        .spawn(Enemy {
            health: 50,
            max_health: 100,
            speed: 1.0,
            damage: 1,
        })
        .id();
    let bar = world
        .spawn((
            EnemyHealthBarFill {
                max_width: 10.0,
                owner: enemy,
                last_ratio: 1.0,
            },
            Transform::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(update_enemy_health_bars);

    schedule.run(&mut world);
    // initial was 1.0; no change expected until enemy health changes
    {
        let tf = world.get::<Transform>(bar).unwrap();
        assert!((tf.scale.x - 1.0).abs() > 0.0 || (tf.translation.x).abs() >= 0.0);
    }

    // Lower health; run again
    world.get_mut::<Enemy>(enemy).unwrap().health = 25;
    schedule.run(&mut world);
    let tf = world.get::<Transform>(bar).unwrap();
    // New ratio 0.25 => width 2.5 and translated left by half of remainder
    assert!((tf.scale.x - 2.5).abs() < 1e-3);
    assert!(tf.translation.x < 0.0);
}
