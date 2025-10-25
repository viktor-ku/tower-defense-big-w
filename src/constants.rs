use bevy::prelude::*;

// App/window
pub const C_WINDOW_TITLE: &str = "Village Defender v0.1";
pub const C_WINDOW_RESOLUTION: (u32, u32) = (1920, 1080);

// Camera and lighting
pub const C_CAMERA_OFFSET_X: f32 = 0.0;
pub const C_CAMERA_OFFSET_Y: f32 = 80.0;
pub const C_CAMERA_OFFSET_Z: f32 = 50.0;
pub const C_LIGHT_ILLUMINANCE: f32 = 10000.0;

// World
pub const C_GROUND_SIZE: f32 = 1000.0;
pub const C_TOWN_SIZE: f32 = 500.0;
pub const C_WALL_THICKNESS: f32 = 4.0;
pub const C_WALL_HEIGHT: f32 = 8.0;
pub const C_GATE_WIDTH: f32 = 20.0;
pub const C_SQUARE_SIZE: f32 = 60.0;
pub const C_GROUND_COLOR_SRGB: (f32, f32, f32) = (0.2, 0.3, 0.2);
pub const C_ROAD_WIDTH: f32 = 5.0;
// Chunking & world seed
pub const C_WORLD_SEED: u64 = 0xC0FFEE_u64;
pub const C_CHUNK_SIZE: f32 = 1024.0;
pub const C_CHUNKS_ACTIVE_RADIUS: i32 = 2; // 5x5 around player
pub const C_CHUNKS_HYSTERESIS: i32 = 1; // keep buffer to avoid thrashing
pub const C_CHUNKS_LOADS_PER_FRAME: usize = 2;
pub const C_CHUNKS_UNLOADS_PER_FRAME: usize = 4;

// Player
pub const C_PLAYER_SPEED: f32 = 80.0;

// Village/base
pub const C_VILLAGE_HEALTH: u32 = 200;
pub const C_VILLAGE_COLLISION_RADIUS: f32 = 8.0;

// Enemies
pub const C_ENEMY_SPAWN_INTERVAL_SECS: f32 = 1.0;
pub const C_ENEMY_DEFAULT_HEALTH: u32 = 60;
pub const C_ENEMY_RANDOM_SPEED_MIN: f32 = 20.0;
pub const C_ENEMY_RANDOM_SPEED_MAX: f32 = 22.0;

// Waves
pub const C_WAVE_INITIAL_DELAY_SECS: f32 = 20.0;
pub const C_WAVE_INTERMISSION_SECS: f32 = 3.0;
pub const C_WAVE_BASE_ENEMY_COUNT: u32 = 10;
pub const C_WAVE_ENEMY_INCREMENT: u32 = 2;
pub const C_WAVE_HEALTH_BONUS_PER_TIER: u32 = 15;

// Towers
pub const C_TOWER_RANGE: f32 = 30.0;
pub const C_TOWER_COST_WOOD: u32 = 5;
pub const C_TOWER_COST_ROCK: u32 = 1;
pub const C_TOWER_SPAWN_EFFECT_DURATION_SECS: f32 = 0.3;
pub const C_PROJECTILE_SPEED: f32 = 80.0;
pub const C_PROJECTILE_HIT_RADIUS: f32 = 1.4;
pub const C_PROJECTILE_LIFETIME_SECS: f32 = 5.0;
pub const C_MAX_BUILD_DISTANCE: f32 = 50.0;
pub const C_RING_INNER_RATIO: f32 = 0.92;
pub const C_IMPACT_EFFECT_DURATION_SECS: f32 = 0.2;
pub const C_DAMAGE_NUMBER_LIFETIME_SECS: f32 = 0.56;
pub const C_DAMAGE_NUMBER_SPAWN_HEIGHT: f32 = 0.0;
pub const C_DAMAGE_NUMBER_FONT_SIZE: f32 = 20.0;
pub const C_ENEMY_FLASH_DURATION_SECS: f32 = 0.20;
pub const C_ENEMY_PRE_EXPLOSION_DURATION_SECS: f32 = 0.6;
pub const C_ENEMY_PRE_EXPLOSION_FLASHES: f32 = 8.0;
pub const C_EXPLOSION_EFFECT_DURATION_SECS: f32 = 0.8;
pub const C_EXPLOSION_EFFECT_MAX_SCALE: f32 = 3.0;

// Projectile trail
pub const C_PROJECTILE_TRAIL_EMIT_INTERVAL_SECS: f32 = 0.1;
pub const C_PROJECTILE_TRAIL_LIFETIME_SECS: f32 = 0.5;
pub const C_PROJECTILE_TRAIL_START_SCALE: f32 = 0.8;
pub const C_PROJECTILE_TRAIL_END_SCALE: f32 = 0.01;

// Health bars (world-space over enemies)
pub const C_HEALTH_BAR_WIDTH: f32 = 4.0;
pub const C_HEALTH_BAR_HEIGHT: f32 = 0.5;
pub const C_HEALTH_BAR_FILL_HEIGHT: f32 = 0.4;
pub const C_HEALTH_BAR_OFFSET_Y: f32 = 4.2;

// Resources placement
pub const C_TREES_COUNT: u32 = 100;
pub const C_TREE_WOOD_MIN: u32 = 20;
pub const C_TREE_WOOD_MAX: u32 = 60;
pub const C_TREE_SIZE: (f32, f32, f32) = (1.4, 3.2, 1.4);

pub const C_ROCKS_COUNT: u32 = 50;
pub const C_ROCK_SIZE: (f32, f32, f32) = (1.0, 0.8, 1.0);

/// Tunable values that control the game. Insert this as a Bevy resource to tweak gameplay,
/// visuals, and pacing without touching system code. Values are read at runtime by systems.
#[derive(Resource, Clone)]
pub struct Tunables {
    /// Overall town square dimension (town_size x town_size)
    pub town_size: f32,
    /// Title of the primary window. Changing this requires a restart/run.
    pub window_title: &'static str,
    /// Window resolution in pixels (width, height). Changing this requires a restart/run.
    pub window_resolution: (u32, u32),

    /// Camera offset from the player in world units (X, Y, Z). Larger Y/Z pulls the camera back.
    pub camera_offset: Vec3,
    /// Directional light illuminance (lux-like units). Higher is brighter.
    pub light_illuminance: f32,

    /// Ground plane dimension (size x size) in world units.
    pub ground_size: f32,
    /// Ground base color (linear sRGB).
    pub ground_color: Color,
    /// Road strip width in world units.
    pub road_width: f32,

    /// Deterministic world seed for procedural content.
    pub world_seed: u64,
    /// Size of a single world chunk (square on XZ plane).
    pub chunk_size: f32,
    /// Active chunk radius around player (Manhattan/Chebyshev based on impl).
    pub chunks_active_radius: i32,
    /// Hysteresis to delay unloading for nearby chunks.
    pub chunks_hysteresis: i32,
    /// Max chunks to load per frame.
    pub chunks_loads_per_frame: usize,
    /// Max chunks to unload per frame.
    pub chunks_unloads_per_frame: usize,

    /// Perimeter wall thickness (X or Z depending on orientation).
    pub wall_thickness: f32,
    /// Perimeter wall height.
    pub wall_height: f32,
    /// Gate opening width on the east wall.
    pub gate_width: f32,
    /// Town square pavement size around the center.
    pub square_size: f32,

    /// Player movement speed in units/second.
    pub player_speed: f32,

    /// Maximum health for the village/base.
    pub village_health: u32,
    /// Collision radius around the village center for enemy impacts.
    pub village_collision_radius: f32,

    /// Seconds between enemy spawns.
    pub enemy_spawn_interval_secs: f32,
    /// Radius of the ring used for random enemy spawns when roads are unavailable.
    pub enemy_spawn_ring_distance: f32,
    /// Default enemy health.
    pub enemy_default_health: u32,
    /// Minimum enemy speed (units/second).
    pub enemy_random_speed_min: f32,
    /// Maximum enemy speed (units/second).
    pub enemy_random_speed_max: f32,
    /// Seconds before the first wave begins.
    pub wave_initial_delay_secs: f32,
    /// Seconds between waves after the first.
    pub wave_intermission_secs: f32,
    /// Base number of enemies spawned during the first wave.
    pub wave_base_enemy_count: u32,
    /// Number of additional enemies added per wave.
    pub wave_enemy_increment: u32,
    /// Enemy health bonus applied per difficulty tier (every 5 waves).
    pub wave_health_bonus_per_tier: u32,

    /// Tower attack range in world units.
    pub tower_range: f32,
    // Tower mesh dimensions removed; sizes are per-kind
    /// Wood cost to build a tower.
    pub tower_cost_wood: u32,
    /// Rock cost to build a tower.
    pub tower_cost_rock: u32,
    /// Seconds for the tower spawn ring effect.
    pub tower_spawn_effect_duration_secs: f32,
    /// Speed of projectiles fired by towers.
    pub projectile_speed: f32,
    /// Radius around the target position considered a hit.
    pub projectile_hit_radius: f32,
    /// Maximum projectile lifetime before self-despawn.
    pub projectile_lifetime_secs: f32,
    /// Maximum distance from the player to place a building.
    pub max_build_distance: f32,
    /// Inner radius ratio for ring meshes (0..1).
    pub ring_inner_ratio: f32,
    /// Duration of the radial impact flash effect.
    pub impact_effect_duration_secs: f32,
    /// Lifetime of floating damage numbers.
    pub damage_number_lifetime_secs: f32,
    /// Initial height offset for damage numbers.
    pub damage_number_spawn_height: f32,
    /// Font size for damage numbers.
    pub damage_number_font_size: f32,
    /// Duration of the white flash applied to enemies on hit.
    pub enemy_flash_duration_secs: f32,
    /// Duration of the pre-explosion warning flash on enemies.
    pub enemy_pre_explosion_duration_secs: f32,
    /// Number of flash cycles during the pre-explosion warning.
    pub enemy_pre_explosion_flashes: f32,
    /// Duration of the spawned explosion effect.
    pub explosion_effect_duration_secs: f32,
    /// Maximum scale of the spawned explosion effect.
    pub explosion_effect_max_scale: f32,

    /// Seconds between trail points emitted by a projectile.
    pub projectile_trail_emit_interval_secs: f32,
    /// Lifetime of a single projectile trail point.
    pub projectile_trail_lifetime_secs: f32,
    /// Initial scale of projectile trail points.
    pub projectile_trail_start_scale: f32,
    /// End scale of projectile trail points.
    pub projectile_trail_end_scale: f32,

    /// Enemy health bar width in world units.
    pub health_bar_width: f32,
    /// Enemy health bar background height in world units.
    pub health_bar_height: f32,
    /// Enemy health bar fill height in world units.
    pub health_bar_fill_height: f32,
    /// Vertical offset above the unit for health bar placement.
    pub health_bar_offset_y: f32,

    /// Number of trees to spawn.
    pub trees_count: u32,
    /// Minimum wood per tree.
    pub tree_wood_min: u32,
    /// Maximum wood per tree.
    pub tree_wood_max: u32,
    /// Tree mesh dimensions.
    pub tree_size: Vec3,

    /// Number of rocks to spawn.
    pub rocks_count: u32,
    /// Rock mesh dimensions.
    pub rock_size: Vec3,
}

impl Default for Tunables {
    fn default() -> Self {
        Tunables {
            town_size: C_TOWN_SIZE,
            // App/window
            window_title: C_WINDOW_TITLE,
            window_resolution: C_WINDOW_RESOLUTION,

            // Camera and lighting
            camera_offset: Vec3::new(C_CAMERA_OFFSET_X, C_CAMERA_OFFSET_Y, C_CAMERA_OFFSET_Z),
            light_illuminance: C_LIGHT_ILLUMINANCE,

            // World
            ground_size: C_GROUND_SIZE,
            ground_color: Color::srgb(
                C_GROUND_COLOR_SRGB.0,
                C_GROUND_COLOR_SRGB.1,
                C_GROUND_COLOR_SRGB.2,
            ),
            road_width: C_ROAD_WIDTH,
            world_seed: C_WORLD_SEED,
            chunk_size: C_CHUNK_SIZE,
            chunks_active_radius: C_CHUNKS_ACTIVE_RADIUS,
            chunks_hysteresis: C_CHUNKS_HYSTERESIS,
            chunks_loads_per_frame: C_CHUNKS_LOADS_PER_FRAME,
            chunks_unloads_per_frame: C_CHUNKS_UNLOADS_PER_FRAME,
            wall_thickness: C_WALL_THICKNESS,
            wall_height: C_WALL_HEIGHT,
            gate_width: C_GATE_WIDTH,
            square_size: C_SQUARE_SIZE,

            // Player
            player_speed: C_PLAYER_SPEED,

            // Village/base
            village_health: C_VILLAGE_HEALTH,
            village_collision_radius: C_VILLAGE_COLLISION_RADIUS,

            // Enemies
            enemy_spawn_interval_secs: C_ENEMY_SPAWN_INTERVAL_SECS,
            enemy_spawn_ring_distance: C_TOWN_SIZE / 2.0 + 100.0,
            enemy_default_health: C_ENEMY_DEFAULT_HEALTH,
            enemy_random_speed_min: C_ENEMY_RANDOM_SPEED_MIN,
            enemy_random_speed_max: C_ENEMY_RANDOM_SPEED_MAX,
            wave_initial_delay_secs: C_WAVE_INITIAL_DELAY_SECS,
            wave_intermission_secs: C_WAVE_INTERMISSION_SECS,
            wave_base_enemy_count: C_WAVE_BASE_ENEMY_COUNT,
            wave_enemy_increment: C_WAVE_ENEMY_INCREMENT,
            wave_health_bonus_per_tier: C_WAVE_HEALTH_BONUS_PER_TIER,

            // Towers
            tower_range: C_TOWER_RANGE,
            tower_cost_wood: C_TOWER_COST_WOOD,
            tower_cost_rock: C_TOWER_COST_ROCK,
            tower_spawn_effect_duration_secs: C_TOWER_SPAWN_EFFECT_DURATION_SECS,
            projectile_speed: C_PROJECTILE_SPEED,
            projectile_hit_radius: C_PROJECTILE_HIT_RADIUS,
            projectile_lifetime_secs: C_PROJECTILE_LIFETIME_SECS,
            max_build_distance: C_MAX_BUILD_DISTANCE,
            ring_inner_ratio: C_RING_INNER_RATIO,
            impact_effect_duration_secs: C_IMPACT_EFFECT_DURATION_SECS,
            damage_number_lifetime_secs: C_DAMAGE_NUMBER_LIFETIME_SECS,
            damage_number_spawn_height: C_DAMAGE_NUMBER_SPAWN_HEIGHT,
            damage_number_font_size: C_DAMAGE_NUMBER_FONT_SIZE,
            enemy_flash_duration_secs: C_ENEMY_FLASH_DURATION_SECS,
            enemy_pre_explosion_duration_secs: C_ENEMY_PRE_EXPLOSION_DURATION_SECS,
            enemy_pre_explosion_flashes: C_ENEMY_PRE_EXPLOSION_FLASHES,
            explosion_effect_duration_secs: C_EXPLOSION_EFFECT_DURATION_SECS,
            explosion_effect_max_scale: C_EXPLOSION_EFFECT_MAX_SCALE,

            // Projectile trail
            projectile_trail_emit_interval_secs: C_PROJECTILE_TRAIL_EMIT_INTERVAL_SECS,
            projectile_trail_lifetime_secs: C_PROJECTILE_TRAIL_LIFETIME_SECS,
            projectile_trail_start_scale: C_PROJECTILE_TRAIL_START_SCALE,
            projectile_trail_end_scale: C_PROJECTILE_TRAIL_END_SCALE,

            // Health bars (world-space)
            health_bar_width: C_HEALTH_BAR_WIDTH,
            health_bar_height: C_HEALTH_BAR_HEIGHT,
            health_bar_fill_height: C_HEALTH_BAR_FILL_HEIGHT,
            health_bar_offset_y: C_HEALTH_BAR_OFFSET_Y,

            // Resources
            trees_count: C_TREES_COUNT,
            tree_wood_min: C_TREE_WOOD_MIN,
            tree_wood_max: C_TREE_WOOD_MAX,
            tree_size: Vec3::new(C_TREE_SIZE.0, C_TREE_SIZE.1, C_TREE_SIZE.2),

            rocks_count: C_ROCKS_COUNT,
            rock_size: Vec3::new(C_ROCK_SIZE.0, C_ROCK_SIZE.1, C_ROCK_SIZE.2),
        }
    }
}
