use rouille::{router, try_or_400, websocket};
use serde::{Deserialize, Serialize};
use std::{fs::File, io, thread, sync::{Mutex, Arc}};

mod serial;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    action: Box<String>,
}

fn main() {
    let serial = Arc::new(Mutex::new(serial::Serial::new("/dev/tty.usbserial-A100JOB2")));

    serial.clone().lock().unwrap().status();

    rouille::start_server("localhost:8000", move |request| {
        rouille::log(&request, io::stdout(), || {
            {
                let response = rouille::match_assets(&request, "./assets");

                if response.is_success() {
                    return response;
                }
            }

            router!(request,
                (GET) (/) => {
                    rouille::Response::from_file("text/html", File::open("assets/index.html").unwrap())
                },

                (GET) (/ws) => {
                    let (response, websocket) = try_or_400!(websocket::start(&request, Some("echo")));

                    let s = serial.clone();
                    thread::spawn(move || {
                        let ws = websocket.recv().unwrap();
                        websocket_handling_thread(ws, s);
                    });

                    response
                },

                _ => rouille::Response::empty_404()
            )
        })
    });
}

fn websocket_handling_thread(mut websocket: websocket::Websocket, serial: Arc<Mutex<serial::Serial>>) {
    websocket.send_text("Hello!").unwrap();

    while let Some(message) = websocket.next() {
        match message {
            websocket::Message::Text(txt) => {
                let message: Message = serde_json::from_str(&txt).unwrap();
                match message.action.as_str() {
                    "volume_up" => serial.lock().unwrap().volume_up(),
                    "volume_down" => serial.lock().unwrap().volume_down(),
                    "turn_on" => serial.lock().unwrap().turn_on(),
                    "turn_off" => serial.lock().unwrap().turn_off(),
                    "mute" => serial.lock().unwrap().mute(),
                    &_ => assert!(false)
                }

                println!("received {:?} from a websocket", message);
                websocket.send_text(&txt).unwrap();
            }
            websocket::Message::Binary(_) => {
                println!("received binary from a websocket");
            }
        }
    }
}
