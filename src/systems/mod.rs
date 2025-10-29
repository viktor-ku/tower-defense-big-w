pub mod camera;
pub mod combat;
pub mod input;
pub mod movement;
pub mod tree_collection;
pub mod ui;
pub mod window;
// world module removed
pub mod chunks;
pub mod resource_passes;
pub mod waves;
// Intentionally avoid re-exporting subsystem items at the root to reduce API bleed.
// Import modules directly, e.g., `use td::systems::waves::wave_progression;`
// Provide a minimal curated prelude for binary crate convenience.
// No re-exports; import directly from submodules.
