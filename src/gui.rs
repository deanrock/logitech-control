use crate::debug::is_debug;
use crate::state::AppState;

use std::sync::Arc;
use std::{
    collections::HashMap,
    fs::{canonicalize, read},
};

use serde::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
use wry::application::platform::macos::{
    ActivationPolicy, CustomMenuItemExtMacOS, EventLoopExtMacOS, NativeImage,
};
#[cfg(target_os = "windows")]
use wry::application::platform::windows::WindowBuilderExtWindows;
use wry::{
    application::{
        accelerator::{Accelerator, SysMods},
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        global_shortcut::ShortcutManager,
        keyboard::KeyCode,
        menu::{ContextMenu, MenuItemAttributes, MenuType},
        system_tray::SystemTrayBuilder,
        window::{WindowBuilder, WindowId},
    },
    http::ResponseBuilder,
    webview::{WebView, WebViewBuilder},
};

#[derive(Serialize, Deserialize, Debug)]
struct ActionMessage {
    action: Box<String>,
}

enum WebViewEvents {
    Message(String),
}

fn get_index() -> String {
    if is_debug() {
        return String::from_utf8_lossy(
            &read(canonicalize(format!("assets/{}", "index.html")).unwrap()).unwrap(),
        )
        .to_string();
    } else {
        return String::from_utf8_lossy(include_bytes!("../assets/index.html")).to_string();
    }
}

fn get_html() -> String {
    let x = String::from_utf8_lossy(include_bytes!("../assets/pure-min.css")).to_string();
    return get_index().replace(
        "<!-- pure-min.css -->",
        format!("<style>{}</style>", x).as_str(),
    );
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn gui(app_state: Arc<AppState>) -> wry::Result<()> {
    // Build our event loop
    #[cfg(target_os = "macos")]
    let mut event_loop = EventLoop::<WebViewEvents>::with_user_event();

    #[cfg(not(target_os = "macos"))]
    let event_loop = EventLoop::<WebViewEvents>::with_user_event();

    // launch macos app without menu and without dock icon
    // should be set at launch
    #[cfg(target_os = "macos")]
    event_loop.set_activation_policy(ActivationPolicy::Accessory);

    let mut webviews: HashMap<WindowId, WebView> = HashMap::new();

    // Create global shortcut
    let mut shortcut_manager = ShortcutManager::new(&event_loop);
    // SysMods::CmdShift; Command + Shift on macOS, Ctrl + Shift on windows/linux
    let my_accelerator = Accelerator::new(SysMods::CmdShift, KeyCode::Digit0);
    let global_shortcut = shortcut_manager.register(my_accelerator.clone()).unwrap();

    // Create sample menu item
    let mut tray_menu = ContextMenu::new();
    let open_new_window = tray_menu.add_item(MenuItemAttributes::new("Open new window"));
    // custom quit who take care to clean windows tray icon
    let quit_item = tray_menu.add_item(MenuItemAttributes::new("Quit"));

    // set NativeImage for `Open new window`
    #[cfg(target_os = "macos")]
    open_new_window
        .clone()
        .set_native_image(NativeImage::StatusAvailable);

    // Windows require Vec<u8> ICO file
    #[cfg(target_os = "windows")]
    let icon = include_bytes!("../assets/icon.ico").to_vec();
    // macOS require Vec<u8> PNG file
    #[cfg(target_os = "macos")]
    let icon = include_bytes!("../assets/icon.png").to_vec();

    // Windows require Vec<u8> ICO file
    #[cfg(target_os = "windows")]
    let new_icon = include_bytes!("../assets/icon_blue.ico").to_vec();
    // macOS require Vec<u8> PNG file

    use tao::window::Window;

    use crate::serial::Effect;
    #[cfg(target_os = "macos")]
    let new_icon = include_bytes!("../assets/icon_dark.png").to_vec();

    let mut system_tray = SystemTrayBuilder::new(icon.clone(), Some(tray_menu))
        .build(&event_loop)
        .unwrap();

    let proxy = event_loop.create_proxy();

    let serial = app_state.serial.clone();

    let handler = move |_: &Window, req: String| {
        println!("{}", req);
        proxy.send_event(WebViewEvents::Message(req));
    };

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        let mut create_window_or_focus = || {
            // if we already have one webview, let's focus instead of opening
            if !webviews.is_empty() {
                for window in webviews.values() {
                    window.window().set_focus();
                }
                return;
            }

            // disable our global shortcut
            shortcut_manager
                .unregister(global_shortcut.clone())
                .unwrap();

            // create our new window / webview instance
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            let window_builder = WindowBuilder::new().with_skip_taskbar(true);
            #[cfg(target_os = "macos")]
            let window_builder = WindowBuilder::new();

            let window = window_builder
                .with_title("Logitech Control")
                .build(event_loop)
                .unwrap();

            let id = window.id();

            let webview = WebViewBuilder::new(window)
                .unwrap()
                .with_ipc_handler(handler.clone())
                .with_custom_protocol("wry".into(), move |request| {
                    /*ResponseBuilder::new()
                    .mimetype("text/html")
                    .body(index_html.as_bytes().into())*/

                    let path = request.uri().replace("wry://", "");
                    let (data, meta) = if path == "assets/index.html" {
                        (get_html().as_bytes().to_vec(), "text/html")
                    } else if path == "assets/favicon.ico" {
                        ("{}".as_bytes().to_vec(), "text/html")
                    } else {
                        println!("{}", path);
                        unimplemented!();
                    };

                    ResponseBuilder::new().mimetype(meta).body(data)
                })
                .with_url("wry://assets/index.html")
                .unwrap()
                .build()
                .unwrap();

            webviews.insert(id, webview);

            // make sure open_new_window is mutable
            let mut open_new_window = open_new_window.clone();
            // disable button
            open_new_window.set_enabled(false);
            // change title (text)
            open_new_window.set_title("Window already open");
            // set checked
            open_new_window.set_selected(true);
            // update tray i  con
            system_tray.set_icon(new_icon.clone());
            // add macOS Native red dot
            #[cfg(target_os = "macos")]
            open_new_window.set_native_image(NativeImage::StatusUnavailable);
        };

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
                ..
            } => {
                println!("Window {:?} has received the signal to close", window_id);
                let mut open_new_window = open_new_window.clone();
                // Remove window from our hashmap
                webviews.remove(&window_id);
                // Modify our button's state
                open_new_window.set_enabled(true);
                // Reset text
                open_new_window.set_title("Open new window");
                // Set selected
                open_new_window.set_selected(false);
                // Change tray icon
                system_tray.set_icon(icon.clone());
                // re-active our global shortcut
                shortcut_manager.register(my_accelerator.clone()).unwrap();
                // macOS have native image available that we can use in our menu-items
                #[cfg(target_os = "macos")]
                open_new_window.set_native_image(NativeImage::StatusAvailable);
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                window_id,
                ..
            } => {
                let _ = webviews[&window_id].resize();
            }
            // on Windows, habitually, we show the Window with left click
            // and the menu is shown on right click
            #[cfg(target_os = "windows")]
            Event::TrayEvent {
                event: tao::event::TrayEvent::LeftClick,
                ..
            } => create_window_or_focus(),
            // Catch menu events
            Event::MenuEvent {
                menu_id,
                // specify only context menu's
                origin: MenuType::ContextMenu,
                ..
            } => {
                // Click on Open new window or focus item
                if menu_id == open_new_window.clone().id() {
                    create_window_or_focus();
                }
                // click on `quit` item
                if menu_id == quit_item.clone().id() {
                    // tell our app to close at the end of the loop.
                    *control_flow = ControlFlow::Exit;
                }
                println!("Clicked on {:?}", menu_id);
            }
            Event::UserEvent(WebViewEvents::Message(req)) => {
                let message: ActionMessage = serde_json::from_str(req.as_str()).unwrap();
                match message.action.as_str() {
                    "volume_up" => serial.lock().unwrap().volume_up(),
                    "volume_down" => serial.lock().unwrap().volume_down(),
                    "turn_on" => serial.lock().unwrap().turn_on(),
                    "turn_off" => serial.lock().unwrap().turn_off(),
                    "mute" => serial.lock().unwrap().mute(),
                    "effect_3d" => serial.lock().unwrap().select_effect(Effect::Effect3d),
                    "effect_2_1" => serial.lock().unwrap().select_effect(Effect::Effect2_1),
                    "effect_4_1" => serial.lock().unwrap().select_effect(Effect::Effect4_1),
                    "effect_disabled" => serial.lock().unwrap().select_effect(Effect::Disabled),
                    &_ => assert!(false),
                }

                let status = serial.lock().unwrap().status();
                let data = serde_json::to_string(&status).unwrap();

                for window in webviews.values() {
                    window
                        .evaluate_script(format!("status('{}')", data).as_str())
                        .unwrap();
                }
            }
            // catch global shortcut event and open window
            Event::GlobalShortcutEvent(hotkey_id) if hotkey_id == my_accelerator.clone().id() => {
                create_window_or_focus()
            }
            _ => (),
        }
    });
}
