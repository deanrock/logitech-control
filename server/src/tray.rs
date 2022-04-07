use crate::state::AppState;
use std::sync::Arc;

use {std::sync::mpsc, tray_item::TrayItem};

enum Message {
    Quit,
}

pub fn tray(app_state: Arc<AppState>) {
    let mut tray = TrayItem::new("Logitech Control", "my-icon-name").unwrap();

    let status = app_state.serial.lock().unwrap().cached_status();

    tray.add_label(format!("Tray Label {:?}", status).as_str()).unwrap();

    tray.add_menu_item("Hello", || {
        println!("Hello!");
    })
    .unwrap();

    let (tx, rx) = mpsc::channel();

    tray.add_menu_item("Quit", move || {
        println!("Quit");
        tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    loop {
        match rx.recv() {
            Ok(Message::Quit) => std::process::exit(0),
            _ => {}
        }
    }
}
