use bevy::prelude::*;

pub fn handle_window_close(
    mut exit_events: MessageWriter<AppExit>,
    mut window_close_events: MessageReader<bevy::window::WindowCloseRequested>,
) {
    for _event in window_close_events.read() {
        info!("Window close requested - forcing immediate exit");
        // Force immediate exit without waiting for cleanup
        std::process::exit(0);
    }
}

pub fn handle_app_exit(mut exit_events: MessageReader<AppExit>) {
    for event in exit_events.read() {
        info!("Application exiting: {:?}", event);
    }
}

// Alternative: Force exit on any window close
pub fn force_exit_on_close(
    _exit_events: MessageWriter<AppExit>,
    mut window_close_events: MessageReader<bevy::window::WindowCloseRequested>,
) {
    for _event in window_close_events.read() {
        info!("Force exit triggered");
        std::process::exit(0);
    }
}
