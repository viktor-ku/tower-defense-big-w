use bevy::prelude::*;

use crate::core::grid::ChunkCoord;
use crate::core::rng::hash_combine;
use rand::{Rng, rngs::StdRng};

pub fn chunk_origin(coord: ChunkCoord, size: f32) -> Vec3 {
    Vec3::new(coord.x as f32 * size, 0.0, coord.z as f32 * size)
}

/// Deterministically generate a per-chunk resource count based on world seed and chunk coordinates.
/// Returns a value in a small band to keep density stable across runs while varying per chunk.
pub fn generate_chunk_resource_count(world_seed: u64, chunk_x: i32, chunk_z: i32) -> u32 {
    // Create a unique seed for this chunk's resource count
    let resource_seed = hash_combine(world_seed ^ 0x1234_5678_9ABC_DEF0, chunk_x, chunk_z);
    // Simple bounded value in [250, 275]
    let mut s = resource_seed;
    // xorshift64*
    s ^= s >> 12;
    s ^= s << 25;
    s ^= s >> 27;
    let val = s.wrapping_mul(0x2545F4914F6CDD1Du64);
    250 + ((val as u32) % 26)
}

#[derive(Debug, Clone, Copy)]
pub enum ExitSide {
    North,
    East,
    South,
    West,
}

/// Choose a random exit side using the given RNG.
pub fn choose_exit_side(rng: &mut StdRng) -> ExitSide {
    match rng.random_range(0..4) {
        0 => ExitSide::North,
        1 => ExitSide::East,
        2 => ExitSide::South,
        _ => ExitSide::West,
    }
}

/// Compute a random lateral offset for a gate along a wall, respecting margins and gate width.
pub fn gate_lateral_offset(
    rng: &mut StdRng,
    half_extent: f32,
    gate_width: f32,
    gate_corner_margin: f32,
) -> f32 {
    let margin = gate_corner_margin.min(half_extent - gate_width * 0.5 - 0.1);
    rng.random_range((-half_extent + margin)..=(half_extent - margin))
}

/// Probability for a tree to be "big" given its distance from the village.
pub fn big_tree_chance(distance_from_village: f32) -> f32 {
    if distance_from_village < 100.0 {
        0.05
    } else if distance_from_village < 200.0 {
        0.20
    } else {
        0.50
    }
}
