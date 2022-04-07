use std::sync::{Arc, Mutex};

use crate::serial;

pub struct AppState {
    pub serial: Arc<Mutex<serial::Serial>>,
}
