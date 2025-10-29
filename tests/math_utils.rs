use bevy::prelude::*;

// Import pure helpers directly from the crate
use td::systems::chunks::world_to_chunk;
use td::systems::resource_passes::{distance_to_polyline_xz, sample_point_on_polyline_xz};

#[test]
fn distance_to_polyline_point_on_segment_is_zero() {
    let path = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 10.0),
    ];
    let p = Vec3::new(5.0, 0.0, 0.0);
    let d = distance_to_polyline_xz(p, &path);
    assert!(d.abs() < 1e-5, "expected zero distance, got {}", d);
}

#[test]
fn distance_to_polyline_closest_to_corner() {
    let path = [Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0)];
    let p = Vec3::new(12.0, 0.0, 3.0);
    let d = distance_to_polyline_xz(p, &path);
    // Closest to (10,0); distance sqrt( (12-10)^2 + 3^2 ) = sqrt(13)
    assert!((d - 13f32.sqrt()).abs() < 1e-5, "got {}", d);
}

#[test]
fn distance_to_polyline_invalid_returns_infinity() {
    let path = [Vec3::new(0.0, 0.0, 0.0)];
    let d = distance_to_polyline_xz(Vec3::ZERO, &path);
    assert!(d.is_infinite());
}

#[test]
fn distance_to_polyline_handles_zero_length_segment() {
    let path = [Vec3::new(1.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)];
    let d = distance_to_polyline_xz(Vec3::new(2.0, 0.0, 1.0), &path);
    assert!((d - 1.0).abs() < 1e-5);
}

#[test]
fn sample_point_on_polyline_start_end() {
    let path = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 10.0),
    ];
    let (p0, dir0) = sample_point_on_polyline_xz(&path, 0.0);
    assert_eq!(p0, path[0]);
    assert_eq!(dir0, Vec3::new(1.0, 0.0, 0.0));

    let (p1, dir1) = sample_point_on_polyline_xz(&path, 1.0);
    assert_eq!(p1, path[2]);
    assert_eq!(dir1, Vec3::new(0.0, 0.0, 1.0));
}

#[test]
fn sample_point_clamps_last_segment() {
    let path = [Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0)];
    let (p, dir) = sample_point_on_polyline_xz(&path, 1.0);
    assert_eq!(p, path[1]);
    assert_eq!(dir, Vec3::new(1.0, 0.0, 0.0));
}

#[test]
fn sample_point_on_polyline_middle_of_second_segment() {
    let path = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 10.0),
    ];
    // 2 segments total; t=0.75 => second segment, local t=0.5 -> (10,0,5)
    let (p, dir) = sample_point_on_polyline_xz(&path, 0.75);
    assert!((p - Vec3::new(10.0, 0.0, 5.0)).length() < 1e-5);
    assert_eq!(dir, Vec3::new(0.0, 0.0, 1.0));
}

#[test]
fn world_to_chunk_basic_mapping() {
    let size = 100.0;
    let c = world_to_chunk(Vec3::new(150.0, 0.0, -120.0), size);
    assert_eq!(c.x, 1);
    assert_eq!(c.z, -2);
}
