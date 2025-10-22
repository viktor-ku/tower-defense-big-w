use bevy::prelude::*;

// Game states
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]

enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
}

// Components
#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Resource {
    resource_type: ResourceType,
    amount: u32,
}

#[derive(Component)]
struct Tower {
    tower_type: TowerType,
    range: f32,
    damage: u32,
    last_shot: f32,
}

#[derive(Component)]
struct Enemy {
    health: u32,
    speed: f32,
    target: Vec3,
}

#[derive(Component)]
struct Village {
    health: u32,
    max_health: u32,
}

#[derive(Component)]
struct DayNight {
    is_day: bool,
    time: f32,
}

#[derive(Component)]
struct BuildingMode {
    active: bool,
    tower_type: TowerType,
}

// Enums
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum ResourceType {
    Wood,
    Rock,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TowerType {
    Basic,
}

// Events
#[derive(Event, Message)]
struct ResourceCollected {
    resource_type: ResourceType,
    amount: u32,
}

#[derive(Event, Message)]
struct TowerBuilt {
    position: Vec3,
    tower_type: TowerType,
}

#[derive(Event, Message)]
struct EnemySpawned {
    position: Vec3,
}

#[derive(Event, Message)]
struct EnemyKilled {
    position: Vec3,
}

// Constants
const VILLAGE_SIZE: f32 = 20.0;
const ROAD_WIDTH: f32 = 2.0;
const TOWER_COST_WOOD: u32 = 5;
const TOWER_COST_ROCK: u32 = 5;
const DAY_DURATION: f32 = 30.0; // 30 seconds per day
const NIGHT_DURATION: f32 = 15.0; // 15 seconds per night

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Village Defender v0.1".into(),
                resolution: (1024, 768).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_message::<ResourceCollected>()
        .add_message::<TowerBuilt>()
        .add_message::<EnemySpawned>()
        .add_message::<EnemyKilled>()
        .add_systems(Startup, setup)
        .add_systems(Update, handle_menu_input.run_if(in_state(GameState::Menu)))
        .add_systems(Update, handle_game_input.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, resource_collection.run_if(in_state(GameState::Playing)))
        .add_systems(Update, tower_building.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy_spawning.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, tower_shooting.run_if(in_state(GameState::Playing)))
        .add_systems(Update, day_night_cycle.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_events.run_if(in_state(GameState::Playing)))
        .run();
}

fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Spawn camera
    commands.spawn(Camera2d::default());
    
    // Spawn player
    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 0.0, 1.0),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Player { speed: 200.0 },
    ));
    
    // Spawn village (center)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.5, 0.3, 0.1),
            custom_size: Some(Vec2::new(VILLAGE_SIZE, VILLAGE_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Village {
            health: 100,
            max_health: 100,
        },
    ));
    
    // Spawn some initial resources
    for i in 0..5 {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / 5.0;
        let distance = 100.0;
        let x = angle.cos() * distance;
        let y = angle.sin() * distance;
        
        let resource_type = if i % 2 == 0 {
            ResourceType::Wood
        } else {
            ResourceType::Rock
        };
        
        let color = match resource_type {
            ResourceType::Wood => Color::srgb(0.6, 0.4, 0.2),
            ResourceType::Rock => Color::srgb(0.4, 0.4, 0.4),
        };
        
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(15.0, 15.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 0.5),
            Resource {
                resource_type,
                amount: 10,
            },
        ));
    }
    
    // Spawn day/night cycle
    commands.spawn((
        DayNight {
            is_day: true,
            time: 0.0,
        },
    ));
    
    // Spawn building mode
    commands.spawn((
        BuildingMode {
            active: false,
            tower_type: TowerType::Basic,
        },
    ));
}

fn handle_menu_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        next_state.set(GameState::Playing);
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

fn handle_game_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut building_mode_query: Query<&mut BuildingMode>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyB) {
        if let Ok(mut building_mode) = building_mode_query.single_mut() {
            building_mode.active = !building_mode.active;
        }
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &Player)>,
) {
    if let Ok((mut transform, player)) = player_query.single_mut() {
        let mut direction = Vec3::ZERO;
        
        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }
        
        if direction.length() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * player.speed * time.delta_secs();
        }
    }
}

fn resource_collection(
    mut commands: Commands,
    player_query: Query<&Transform, (With<Player>, Without<Resource>)>,
    resource_query: Query<(Entity, &Transform, &Resource)>,
    mut resource_events: MessageWriter<ResourceCollected>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (entity, resource_transform, resource) in resource_query.iter() {
            let distance = player_transform
                .translation
                .distance(resource_transform.translation);
            
            if distance < 25.0 {
                resource_events.write(ResourceCollected {
                    resource_type: resource.resource_type,
                    amount: resource.amount,
                });
                commands.entity(entity).despawn();
            }
        }
    }
}

fn tower_building(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    player_query: Query<&Transform, (With<Player>, Without<Tower>)>,
    mut building_mode_query: Query<&mut BuildingMode>,
    mut tower_events: MessageWriter<TowerBuilt>,
) {
    if let Ok(mut building_mode) = building_mode_query.single_mut() {
        if !building_mode.active {
            return;
        }
        
        if mouse_input.just_pressed(MouseButton::Left) {
            if let Ok(window) = windows.single() {
                if let Some(cursor_position) = window.cursor_position() {
                    if let Ok((camera, camera_transform)) = camera_query.single() {
                        if let Ok(world_position) = camera.viewport_to_world_2d(
                            camera_transform,
                            cursor_position,
                        ) {
                            // Check if position is valid (not too close to village or other towers)
                            let village_distance = world_position.distance(Vec2::ZERO);
                            if village_distance > VILLAGE_SIZE / 2.0 + 30.0 {
                                // Check distance from player
                                if let Ok(player_transform) = player_query.single() {
                                    let player_distance = world_position
                                        .distance(player_transform.translation.truncate());
                                    if player_distance < 50.0 {
                                        // Build tower
                                        let tower_position = world_position.extend(0.5);
                                        
                                        commands.spawn((
                                            Sprite {
                                                color: Color::srgb(0.3, 0.3, 0.3),
                                                custom_size: Some(Vec2::new(25.0, 25.0)),
                                                ..default()
                                            },
                                            Transform::from_translation(tower_position),
                                            Tower {
                                                tower_type: building_mode.tower_type,
                                                range: 80.0,
                                                damage: 25,
                                                last_shot: 0.0,
                                            },
                                        ));
                                        
                                        tower_events.write(TowerBuilt {
                                            position: tower_position,
                                            tower_type: building_mode.tower_type,
                                        });
                                        
                                        building_mode.active = false;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn enemy_spawning(
    mut commands: Commands,
    time: Res<Time>,
    mut enemy_events: MessageWriter<EnemySpawned>,
) {
    // Simple enemy spawning - spawn one every 3 seconds
    if time.elapsed_secs_f64() as f32 % 3.0 < time.delta_secs() {
        let angle = (time.elapsed_secs_f64() as f32 * 0.5) % (2.0 * std::f32::consts::PI);
        let distance = 200.0;
        let x = angle.cos() * distance;
        let y = angle.sin() * distance;
        let position = Vec3::new(x, y, 0.5);
        
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(15.0, 15.0)),
                ..default()
            },
            Transform::from_translation(position),
            Enemy {
                health: 50,
                speed: 30.0,
                target: Vec3::ZERO, // Target village
            },
        ));
        
        enemy_events.write(EnemySpawned { position });
    }
}

fn enemy_movement(
    time: Res<Time>,
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    mut village_query: Query<&mut Village>,
) {
    for (mut transform, enemy) in enemy_query.iter_mut() {
        // Move towards village
        let direction = (Vec3::ZERO - transform.translation).normalize();
        transform.translation += direction * enemy.speed * time.delta_secs();
        
        // Check if reached village
        if transform.translation.distance(Vec3::ZERO) < VILLAGE_SIZE / 2.0 {
            if let Ok(mut village) = village_query.single_mut() {
                village.health = village.health.saturating_sub(10);
            }
            // Despawn enemy (this would be handled by a separate system in a real game)
        }
    }
}

fn tower_shooting(
    time: Res<Time>,
    mut tower_query: Query<(&Transform, &mut Tower)>,
    enemy_query: Query<(&Transform, &mut Enemy)>,
    mut enemy_killed_events: MessageWriter<EnemyKilled>,
) {
    for (tower_transform, mut tower) in tower_query.iter_mut() {
        tower.last_shot += time.delta_secs();
        
        if tower.last_shot >= 1.0 {
            // Find closest enemy in range
            let mut closest_enemy = None;
            let mut closest_distance = f32::MAX;
            
            for (enemy_transform, enemy) in enemy_query.iter() {
                let distance = tower_transform
                    .translation
                    .distance(enemy_transform.translation);
                
                if distance <= tower.range && distance < closest_distance {
                    closest_enemy = Some((enemy_transform, enemy));
                    closest_distance = distance;
                }
            }
            
            if let Some((enemy_transform, _enemy)) = closest_enemy {
                // Shoot enemy - we need to get a mutable reference
                // For now, let's just mark the tower as having shot
                tower.last_shot = 0.0;
                
                // TODO: Implement proper enemy health reduction
                // This would require a different approach to get mutable access
                enemy_killed_events.write(EnemyKilled {
                    position: enemy_transform.translation,
                });
            }
        }
    }
}

fn day_night_cycle(
    time: Res<Time>,
    mut day_night_query: Query<&mut DayNight>,
) {
    if let Ok(mut day_night) = day_night_query.single_mut() {
        day_night.time += time.delta_secs();
        
        let cycle_duration = if day_night.is_day {
            DAY_DURATION
        } else {
            NIGHT_DURATION
        };
        
        if day_night.time >= cycle_duration {
            day_night.is_day = !day_night.is_day;
            day_night.time = 0.0;
        }
    }
}

fn handle_events(
    mut resource_events: MessageReader<ResourceCollected>,
    mut tower_events: MessageReader<TowerBuilt>,
    mut enemy_spawned_events: MessageReader<EnemySpawned>,
    mut enemy_killed_events: MessageReader<EnemyKilled>,
) {
    for event in resource_events.read() {
        info!("Resource collected: {:?} x{}", event.resource_type, event.amount);
    }
    
    for event in tower_events.read() {
        info!("Tower built at: {:?}", event.position);
    }
    
    for event in enemy_spawned_events.read() {
        info!("Enemy spawned at: {:?}", event.position);
    }
    
    for event in enemy_killed_events.read() {
        info!("Enemy killed at: {:?}", event.position);
    }
}