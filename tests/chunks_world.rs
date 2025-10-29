use bevy::prelude::*;
use td::systems::chunks::world_to_chunk;

#[test]
fn world_to_chunk_on_boundaries() {
    let size = 128.0;
    let c0 = world_to_chunk(Vec3::new(0.0, 0.0, 0.0), size);
    assert_eq!((c0.x, c0.z), (0, 0));

    let c1 = world_to_chunk(Vec3::new(127.999, 0.0, 127.999), size);
    assert_eq!((c1.x, c1.z), (0, 0));

    let c2 = world_to_chunk(Vec3::new(128.0, 0.0, 0.0), size);
    assert_eq!((c2.x, c2.z), (1, 0));

    let c3 = world_to_chunk(Vec3::new(-0.001, 0.0, -0.001), size);
    assert_eq!((c3.x, c3.z), (-1, -1));
}
