use std::sync::{Mutex, Arc};

use tokio::sync::broadcast;

use crate::serial;

pub struct AppState {
    pub serial: Arc<Mutex<serial::Serial>>,
    pub tx: broadcast::Sender<String>,
}
