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

pub use assets::{CombatVfxAssets, EnemyHealthBarAssets};
pub use enemy::{
    cleanup_enemy_health_bars, enemy_spawning, face_enemy_health_bars, position_enemy_health_bars,
    update_enemy_health_bars,
};
pub use projectiles::{
    damage_dealt_spawn_text_system, damage_number_system, enemy_fade_out_system,
    enemy_flash_system, impact_effect_system, projectile_system, tower_shooting,
};
pub use towers::{tower_building, tower_selling_click, tower_spawn_effect_system};
