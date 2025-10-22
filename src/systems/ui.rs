use bevy::prelude::*;
use crate::events::*;

pub fn handle_events(
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
