use bevy::prelude::*;
use rand::{Rng, rngs::StdRng};

#[derive(Debug, Clone, Copy)]
pub enum RoadPattern {
    Straight,
    Curved,
    Snake,
}

/// Generates a random road path (straight, curved, snake) between two points.
pub fn generate_road_pattern(
    start: Vec3,
    end: Vec3,
    _width: f32,
    rng: &mut StdRng,
) -> Option<Vec<Vec3>> {
    let pattern = match rng.random_range(0..3) {
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
            let wiggle_amplitude = 6.0 + rng.random::<f32>() * 8.0; // 6-14 units
            let wiggle_frequency = 2.0 + rng.random::<f32>() * 3.0; // 2-5 waves
            let phase_offset = rng.random::<f32>() * 2.0 * std::f32::consts::PI; // Random phase

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
                let noise_x = (rng.random::<f32>() - 0.5) * noise_amplitude;
                let noise_z = (rng.random::<f32>() - 0.5) * noise_amplitude;

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
            let curve_strength = 20.0 + rng.random::<f32>() * 40.0; // 20-60 units
            let mid1_offset = 0.2 + rng.random::<f32>() * 0.3; // 0.2-0.5
            let mid2_offset = 0.5 + rng.random::<f32>() * 0.3; // 0.5-0.8

            let mid1 = start
                + (end - start) * mid1_offset
                + Vec3::new(
                    (rng.random::<f32>() - 0.5) * curve_strength,
                    0.0,
                    (rng.random::<f32>() - 0.5) * curve_strength,
                );
            let mid2 = start
                + (end - start) * mid2_offset
                + Vec3::new(
                    (rng.random::<f32>() - 0.5) * curve_strength,
                    0.0,
                    (rng.random::<f32>() - 0.5) * curve_strength,
                );

            let segments = 15 + (rng.random::<u8>() % 11) as usize; // 15-25 segments
            generate_bezier_curve(start, mid1, mid2, end, segments)
        }
        RoadPattern::Snake => {
            // S-shaped road with random variations
            let mut waypoints = Vec::new();
            let steps = 25 + (rng.random::<u8>() % 16) as usize; // 25-40 steps

            // Random variations for snake pattern
            let snake_amplitude = 20.0 + rng.random::<f32>() * 25.0; // 20-45 units
            let snake_frequency = 1.5 + rng.random::<f32>() * 2.0; // 1.5-3.5 waves
            let phase_offset = rng.random::<f32>() * 2.0 * std::f32::consts::PI;

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
pub fn generate_bezier_curve(
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

/// Compute tiling for a road segment from `last` to `current` with a desired target patch length.
/// Returns (patch_count, patch_len, forward_dir_normalized, yaw_radians).
pub fn segment_patch_tiling(
    last: Vec3,
    current: Vec3,
    target_patch_len: f32,
) -> Option<(u32, f32, Vec3, f32)> {
    let dir = current - last;
    let seg_len = dir.length();
    if seg_len <= 0.001 {
        return None;
    }
    let yaw = dir.z.atan2(dir.x);
    let forward = dir / seg_len;
    let patch_count = (seg_len / target_patch_len).ceil().max(1.0) as u32;
    let patch_len = seg_len / patch_count as f32;
    Some((patch_count, patch_len, forward, yaw))
}
