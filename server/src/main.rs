use std::{
    sync::{Arc, Mutex},
    thread,
};
use tokio::{
    runtime::Handle,
    sync::broadcast::{self},
};

mod gui;
mod serial;
mod server;
mod state;

fn get_serial() -> Option<serial::Serial> {
    if let Some(port) = serial::find_port() {
        let mut serial = serial::Serial::new(port);

        serial.status();
        return Some(serial);
    }

    return None;
}

#[tokio::main]
async fn main() {
    if let Some(serial) = get_serial() {
        let serial = Arc::new(Mutex::new(serial));
        let (tx, _rx) = broadcast::channel(100);
        let app_state = Arc::new(state::AppState { serial, tx });

        let handle = Handle::current();
        thread::spawn(move || {
            handle.spawn(async move { server::server(app_state).await });
        });

        gui::gui();
    }

    assert!(false, "Cannot find serial device!");
}
