//! Entity construction helpers and bundles.
//!
//! Use this module to define how complex game objects are spawned from
//! components. Keeping construction in one place makes it easy to change
//! visuals or attached components without touching gameplay systems.
//!
//! Tips for contributors:
//! - Prefer `Bundle`s for common spawn sets (e.g. Enemy, Tower, Village)
//! - Expose small helper functions like `spawn_enemy(...)` that return `Entity`
//! - Keep systems focused on behavior; call helpers here to construct entities
//!
//! Example outline (fill in as needed):
//! ```ignore
//! #[derive(Bundle)]
//! pub struct EnemyBundle { /* components... */ }
//!
//! pub fn spawn_enemy(commands: &mut Commands, /* assets, config, etc */) -> Entity { /* ... */ }
//! ```


