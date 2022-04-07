use std::sync::{Arc, Mutex};

mod gui;
mod serial;
mod state;

fn get_serial() -> Option<serial::Serial> {
    if let Some(port) = serial::find_port() {
        let mut serial = serial::Serial::new(port);

        serial.status();
        return Some(serial);
    }

    return None;
}

fn main() {
    let serial = get_serial().unwrap();
    let serial = Arc::new(Mutex::new(serial));
    let app_state = Arc::new(state::AppState { serial });

    gui::gui(app_state).unwrap();
}
