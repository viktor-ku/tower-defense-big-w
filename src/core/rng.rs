/// Mix a base seed with two counters for derived deterministic seeds.
pub fn derive_seed(base: u64, a: u64, b: u64) -> u64 {
    let mut h = base ^ 0x9E37_79B9_7F4A_7C15u64;
    h ^= a.wrapping_mul(0xC2B2_AE3D_27D4_EB4Fu64);
    h = h.rotate_left(27) ^ (h >> 33);
    h ^= b.wrapping_mul(0x1656_67B1_9E3_779F9u64);
    h ^ (h >> 29)
}

/// Combine a seed with 2D integer coordinates into a 64-bit seed.
pub fn hash_combine(seed: u64, x: i32, z: i32) -> u64 {
    let mut h = seed ^ 0x9E37_79B9_7F4A_7C15u64;
    h ^= (x as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4Fu64);
    h = h.rotate_left(27) ^ (h >> 33);
    h ^= (z as u64).wrapping_mul(0x1656_67B1_9E37_79F9u64);
    h ^ (h >> 29)
}

use rand::{Rng, rngs::StdRng};

#[inline]
pub fn pick_f32(
    seeded: bool,
    seeded_rng: &mut StdRng,
    thread_rng: &mut rand::rngs::ThreadRng,
) -> f32 {
    if seeded {
        seeded_rng.random::<f32>()
    } else {
        thread_rng.random::<f32>()
    }
}

#[inline]
pub fn pick_u32(
    seeded: bool,
    seeded_rng: &mut StdRng,
    thread_rng: &mut rand::rngs::ThreadRng,
) -> u32 {
    if seeded {
        seeded_rng.random::<u32>()
    } else {
        thread_rng.random::<u32>()
    }
}
