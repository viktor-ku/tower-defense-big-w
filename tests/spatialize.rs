use bevy::prelude::*;

use td::audio::{SpatialAudioParams, spatialize};

fn listener_at(x: f32, y: f32, z: f32, yaw_rad: f32) -> GlobalTransform {
    GlobalTransform::from(Transform {
        translation: Vec3::new(x, y, z),
        rotation: Quat::from_rotation_y(yaw_rad),
        ..Default::default()
    })
}

#[test]
fn spatialize_center_front_has_pan_zero() {
    let listener = listener_at(0.0, 0.0, 0.0, 0.0);
    let (vol, pan) = spatialize(
        Vec3::new(0.0, 0.0, 10.0),
        &listener,
        SpatialAudioParams::default(),
    );
    assert!(vol > 0.0 && vol <= 1.0);
    assert!(pan.abs() < 1e-5);
}

#[test]
fn spatialize_full_left_right_panning() {
    let listener = listener_at(0.0, 0.0, 0.0, 0.0);
    let (_, pan_left) = spatialize(
        Vec3::new(-10.0, 0.0, 0.0),
        &listener,
        SpatialAudioParams::default(),
    );
    let (_, pan_right) = spatialize(
        Vec3::new(10.0, 0.0, 0.0),
        &listener,
        SpatialAudioParams::default(),
    );
    assert!(pan_left < -0.9);
    assert!(pan_right > 0.9);
}

#[test]
fn spatialize_respects_listener_yaw() {
    // Rotate listener 90deg, so front becomes +X
    let listener = listener_at(0.0, 0.0, 0.0, std::f32::consts::FRAC_PI_2);
    let (_, pan_front) = spatialize(
        Vec3::new(10.0, 0.0, 0.0),
        &listener,
        SpatialAudioParams::default(),
    );
    assert!(
        pan_front.abs() < 0.1,
        "front should have near-zero pan with rotated listener"
    );
}

#[test]
fn spatialize_volume_clamps_beyond_max_distance() {
    let listener = listener_at(0.0, 0.0, 0.0, 0.0);
    let params = SpatialAudioParams {
        attenuation: 0.1,
        max_audible_distance: 50.0,
    };
    let (v_near, _) = spatialize(Vec3::new(0.0, 0.0, 10.0), &listener, params);
    let (v_far, _) = spatialize(Vec3::new(0.0, 0.0, 1000.0), &listener, params);
    assert!(v_near > 0.0);
    assert_eq!(v_far, 0.0);
}
