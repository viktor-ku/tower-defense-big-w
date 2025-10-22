use bevy::prelude::*;

// Force exit on any window close
pub fn force_exit_on_close(
    mut window_close_events: MessageReader<bevy::window::WindowCloseRequested>,
) {
    for _event in window_close_events.read() {
        info!("Force exit triggered");
        std::process::exit(0);
    }
}
