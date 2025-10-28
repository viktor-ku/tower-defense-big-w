//! Components and ECS resources used across the game.
//!
//! This module is split into small files by domain to make it easy to find
//! and extend things. If you're adding a new gameplay concept, add a new file
//! and re-export it here so other modules can `use crate::components::*;`.
//!
//! Layout:
//! - state.rs: global game state enums
//! - player.rs: player components and markers
//! - harvesting.rs: resource nodes and collection state
//! - towers.rs: tower components and preview (ghost)
//! - enemies.rs: enemy components and health bar data
//! - town.rs: town, walls, gates, and building mode flag
//! - roads.rs: road paths and path-following helpers
//! - chunks.rs: chunk markers
//! - waves.rs: wave progression data/state

pub mod chunks;
pub mod enemies;
pub mod harvesting;
pub mod player;
pub mod render;
pub mod roads;
pub mod state;
pub mod towers;
pub mod town;
pub mod waves;

// Re-export everything for ergonomic wildcard imports in systems
pub use chunks::*;
pub use enemies::*;
pub use harvesting::*;
pub use player::*;
pub use render::*;
pub use roads::*;
pub use state::*;
pub use towers::*;
pub use town::*;
pub use waves::*;
