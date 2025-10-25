use bevy::prelude::*;

/// Forces process exit when a window close is requested.
pub fn force_exit_on_close(
    mut window_close_events: MessageReader<bevy::window::WindowCloseRequested>,
) {
    if let Some(_event) = window_close_events.read().next() {
        info!("Force exit triggered");
        std::process::exit(0);
    }
}
