use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    handler::Handler,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Extension, Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serial::Serial;
use std::{
    fs::{self},
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast::{self, Sender};
use tower_http::services::ServeDir;

mod serial;

#[derive(Serialize, Deserialize, Debug)]
struct ActionMessage {
    action: Box<String>,
}

struct AppState {
    serial: Arc<Mutex<serial::Serial>>,
    tx: broadcast::Sender<String>,
}

fn get_serial() -> Option<serial::Serial> {
    if let Some(port) = serial::find_port() {
        let mut serial = serial::Serial::new(port);

        serial.status();
        return Some(serial);
    }

    return None;
}

async fn server(app_state: Arc<AppState>) {
    let app = Router::new()
        .nest(
            "/assets",
            get_service(ServeDir::new("./assets")).handle_error(
                |error: std::io::Error| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    )
                },
            ),
        )
        .route("/", get(index))
        .route("/ws", get(websocket_handler))
        .fallback(handler_404.into_service())
        .layer(Extension(app_state));

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    if let Some(serial) = get_serial() {
        let serial = Arc::new(Mutex::new(serial));
        let (tx, _rx) = broadcast::channel(100);
        let app_state = Arc::new(AppState { serial, tx });
        return server(app_state).await;
    }

    assert!(false, "Cannot find serial device!");
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 not found")
}

async fn index() -> Html<String> {
    let contents = fs::read_to_string("./assets/index.html").unwrap();
    Html(contents)
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn report_status(serial: &Arc<Mutex<Serial>>, tx: &Sender<String>) {
    let status = serial.lock().unwrap().status();

    let data = serde_json::to_string(&status).unwrap();
    tx.send(data).unwrap();
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();

    let mut rx = state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();
    let serial = state.serial.clone();
    
    report_status(&serial, &tx).await;

    let mut recv_task = tokio::spawn(async move {
        // ping not handled here, since we are using `stream.split()` :/
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Text(text) = message {
                let message: ActionMessage = serde_json::from_str(text.as_str()).unwrap();
                match message.action.as_str() {
                    "volume_up" => serial.lock().unwrap().volume_up(),
                    "volume_down" => serial.lock().unwrap().volume_down(),
                    "turn_on" => serial.lock().unwrap().turn_on(),
                    "turn_off" => serial.lock().unwrap().turn_off(),
                    "mute" => serial.lock().unwrap().mute(),
                    &_ => assert!(false),
                }

                serial.lock().unwrap().status();

                report_status(&serial, &tx).await;

                println!("received {:?} from a websocket", message);
            } else {
                println!("recv {:?}", message);
                //sender.send(Message::Text(String::from("pong"))).await;
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
