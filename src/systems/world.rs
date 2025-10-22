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

// Map world y to z so lower y draws above (fake 3D layering)
pub fn y_to_z_sort(mut q: Query<&mut Transform, With<YSort>>) {
    for mut transform in q.iter_mut() {
        let y = transform.translation.y;
        // Compress into a tight z band to preserve UI/camera layers
        let z = 0.5 - (y / 10000.0).clamp(-0.49, 0.49);
        transform.translation.z = z;
    }
}
