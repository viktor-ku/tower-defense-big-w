//! Combat-related systems split into focused submodules.
//!
//! Modules:
//! - `assets`: reusable mesh/material caches for combat visuals
//! - `towers`: tower placement logic and spawn effects
//! - `enemy`: enemy spawning and health bar maintenance
//! - `projectiles`: tower attacks, projectile movement, and VFX clean-up

pub mod assets;
pub mod enemy;
pub mod projectiles;
pub mod towers;
