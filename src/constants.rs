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
pub const C_WALL_THICKNESS: f32 = 6.0;
pub const C_WALL_HEIGHT: f32 = 6.0;
pub const C_GATE_WIDTH: f32 = 20.0;
pub const C_SQUARE_SIZE: f32 = 80.0;
pub const C_GROUND_COLOR_SRGB: (f32, f32, f32) = (0.2, 0.35, 0.2);
pub const C_ROAD_WIDTH: f32 = 6.0;
pub const C_ROAD_ENDPOINT_DISTANCE: f32 = 100.0;

// Player
pub const C_PLAYER_SPEED: f32 = 80.0;

// Village/base
pub const C_VILLAGE_HEALTH: u32 = 100;
pub const C_VILLAGE_COLLISION_RADIUS: f32 = 5.0;

// Enemies
pub const C_ENEMY_SPAWN_INTERVAL_SECS: f32 = 3.0;
// Will be overridden in Tunables::default to C_TOWN_SIZE/2 + 100.0
pub const C_ENEMY_SPAWN_RING_DISTANCE: f32 = 200.0;
pub const C_ENEMY_DEFAULT_HEALTH: u32 = 50;
pub const C_ENEMY_RANDOM_SPEED_MIN: f32 = 10.0;
pub const C_ENEMY_RANDOM_SPEED_MAX: f32 = 25.0;

// Towers
pub const C_TOWER_RANGE: f32 = 45.0;
pub const C_TOWER_WIDTH: f32 = 1.2;
pub const C_TOWER_HEIGHT: f32 = 3.2;
pub const C_TOWER_DEPTH: f32 = 1.2;
pub const C_TOWER_DAMAGE: u32 = 25;
pub const C_TOWER_FIRE_INTERVAL_SECS: f32 = 1.0;
pub const C_TOWER_SPAWN_EFFECT_DURATION_SECS: f32 = 0.45;
pub const C_MAX_BUILD_DISTANCE: f32 = 100.0;
pub const C_RING_INNER_RATIO: f32 = 0.92;

// Health bars (world-space over enemies)
pub const C_HEALTH_BAR_WIDTH: f32 = 3.0;
pub const C_HEALTH_BAR_HEIGHT: f32 = 0.28;
pub const C_HEALTH_BAR_FILL_HEIGHT: f32 = 0.2;
pub const C_HEALTH_BAR_OFFSET_Y: f32 = C_TOWER_HEIGHT + 0.8;

// Resources placement
pub const C_TREES_COUNT: u32 = 80;
pub const C_TREE_WOOD_MIN: u32 = 15;
pub const C_TREE_WOOD_MAX: u32 = 35;
pub const C_TREE_SIZE: (f32, f32, f32) = (1.2, 3.0, 1.2);
pub const C_TREE_DISTANCE_MIN: f32 = 60.0;
pub const C_TREE_DISTANCE_MAX: f32 = 210.0; // inside walls

pub const C_ROCKS_COUNT: u32 = 40;
pub const C_ROCK_SIZE: (f32, f32, f32) = (0.8, 0.6, 0.8);
pub const C_ROCK_DISTANCE_MIN: f32 = 55.0;
pub const C_ROCK_DISTANCE_MAX: f32 = 190.0; // inside walls

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
    /// Distance from the center where roads start/end.
    pub road_endpoint_distance: f32,

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

    /// Tower attack range in world units.
    pub tower_range: f32,
    /// Tower mesh dimensions (width, height, depth).
    pub tower_width: f32,
    pub tower_height: f32,
    pub tower_depth: f32,
    /// Damage per shot.
    pub tower_damage: u32,
    /// Seconds between tower shots.
    pub tower_fire_interval_secs: f32,
    /// Seconds for the tower spawn ring effect.
    pub tower_spawn_effect_duration_secs: f32,
    /// Maximum distance from the player to place a building.
    pub max_build_distance: f32,
    /// Inner radius ratio for ring meshes (0..1).
    pub ring_inner_ratio: f32,

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
    /// Minimum radial distance for spawning trees.
    pub tree_distance_min: f32,
    /// Maximum radial distance for spawning trees.
    pub tree_distance_max: f32,

    /// Number of rocks to spawn.
    pub rocks_count: u32,
    /// Rock mesh dimensions.
    pub rock_size: Vec3,
    /// Minimum radial distance for spawning rocks.
    pub rock_distance_min: f32,
    /// Maximum radial distance for spawning rocks.
    pub rock_distance_max: f32,
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
            road_endpoint_distance: C_ROAD_ENDPOINT_DISTANCE,
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

            // Towers
            tower_range: C_TOWER_RANGE,
            tower_width: C_TOWER_WIDTH,
            tower_height: C_TOWER_HEIGHT,
            tower_depth: C_TOWER_DEPTH,
            tower_damage: C_TOWER_DAMAGE,
            tower_fire_interval_secs: C_TOWER_FIRE_INTERVAL_SECS,
            tower_spawn_effect_duration_secs: C_TOWER_SPAWN_EFFECT_DURATION_SECS,
            max_build_distance: C_MAX_BUILD_DISTANCE,
            ring_inner_ratio: C_RING_INNER_RATIO,

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
            tree_distance_min: C_TREE_DISTANCE_MIN,
            tree_distance_max: C_TREE_DISTANCE_MAX,

            rocks_count: C_ROCKS_COUNT,
            rock_size: Vec3::new(C_ROCK_SIZE.0, C_ROCK_SIZE.1, C_ROCK_SIZE.2),
            rock_distance_min: C_ROCK_DISTANCE_MIN,
            rock_distance_max: C_ROCK_DISTANCE_MAX,
        }
    }
}
