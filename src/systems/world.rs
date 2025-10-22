use bevy::prelude::*;
use crate::components::*;

pub fn day_night_cycle(
    time: Res<Time>,
    mut day_night_query: Query<&mut DayNight>,
) {
    for mut day_night in day_night_query.iter_mut() {
        day_night.time_until_switch -= time.delta_secs();
        
        if day_night.time_until_switch <= 0.0 {
            day_night.is_day = !day_night.is_day;
            day_night.time_until_switch = if day_night.is_day {
                day_night.day_duration
            } else {
                day_night.night_duration
            };
            
            info!("Time of day: {}", if day_night.is_day { "Day" } else { "Night" });
        }
    }
}

