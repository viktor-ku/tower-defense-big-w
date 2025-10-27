use crate::events::*;
use bevy::prelude::*;

// Observer-based logging for gameplay events (Bevy 0.17)
pub fn on_resource_collected(trigger: On<ResourceCollected>) {
    let e = trigger.event();
    if cfg!(debug_assertions) {
        info!("Resource collected: {:?} x{}", e.kind, e.amount);
    }
}

pub fn on_tower_built(trigger: On<TowerBuilt>) {
    let e = trigger.event();
    if cfg!(debug_assertions) {
        info!("Tower built at: {:?}", e.position);
    }
}

pub fn on_enemy_spawned(trigger: On<EnemySpawned>) {
    let e = trigger.event();
    if cfg!(debug_assertions) {
        info!("Enemy spawned at: {:?}", e.position);
    }
}

pub fn on_enemy_killed(trigger: On<EnemyKilled>) {
    let e = trigger.event();
    if cfg!(debug_assertions) {
        info!("Enemy killed at: {:?}", e.position);
    }
}
