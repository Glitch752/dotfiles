use std::sync::Mutex;
use niri_ipc::{Request, Response, socket::Socket};
use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};
use zbus::fdo::PropertiesProxy;

use crate::{networkmanager::get_networkmanager_state, niri::niri_request, systemtray::get_systray_items, upower::{create_upower_proxy, get_upower_properties}};

mod upower;
mod networkmanager;
mod systemtray;
mod niri;

struct BarHandler {
    socket: Option<Socket>,
    upower_proxy: Option<PropertiesProxy<'static>>
}

impl BarHandler {
    pub async fn new() -> Self {
        let mut socket = Socket::connect().ok();

        if let Some(ref mut socket) = socket {
            // Start an event stream with Niri
            if !matches!(socket.send(Request::EventStream), Ok(Ok(Response::Handled))) {
                eprintln!("Failed to start Niri event stream");
            }
        } else {
            eprintln!("Failed to connect to Niri IPC!");
        }

        let upower_proxy = match create_upower_proxy().await {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("Failed to create UPower proxy: {}", e);
                None
            }
        };

        Self {
            socket,
            upower_proxy
        }
    }
}

#[tauri::command]
fn debug_log(msg: String) {
    println!("{}", msg);
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("bar")
        .invoke_handler(tauri::generate_handler![
            debug_log,
            niri_request,
            get_upower_properties,
            get_networkmanager_state,
            get_systray_items
        ])
        .setup(|app, _plugin_api| {
            let app_ = app.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut handler = BarHandler::new().await;
                
                handler
                    .start_niri_event_thread(&app_)
                    .expect("Failed to start Niri event listener thread");

                handler.start_upower_events(&app_);

                handler.start_networkmanager_events(&app_);

                handler.start_system_tray_events(&app_).await;

                app_.manage(Mutex::new(handler));
            });

            Ok(())
        })
        .build()
}
