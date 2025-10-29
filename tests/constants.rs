use bevy::prelude::*;
use td::constants::{C_RING_INNER_RATIO, C_TOWER_RANGE, C_TOWN_SIZE, Tunables};

#[test]
fn tunables_defaults_are_consistent() {
    let t = Tunables::default();
    assert_eq!(t.window_title, "Village Defender v0.1");
    assert_eq!(t.camera_offset, Vec3::new(0.0, 80.0, 50.0));
    assert_eq!(t.tower_range, C_TOWER_RANGE);
    assert_eq!(t.ring_inner_ratio, C_RING_INNER_RATIO);
    assert!((t.enemy_spawn_ring_distance - (C_TOWN_SIZE / 2.0 + 100.0)).abs() < 1e-5);
}
