use std::{sync::Mutex, thread};

use niri_ipc::{Request, Response, socket::Socket};
use tauri::{
    AppHandle, Emitter, Manager, Runtime, State,
    plugin::{Builder, TauriPlugin},
};

struct BarHandler {
    socket: Option<Socket>,
}

impl BarHandler {
    pub fn new() -> Self {
        let mut socket = Socket::connect().ok();

        if let Some(ref mut socket) = socket {
            // Start an event stream with Niri
            if !matches!(socket.send(Request::EventStream), Ok(Ok(Response::Handled))) {
                eprintln!("Failed to start Niri event stream");
            }
        } else {
            eprintln!("Failed to connect to Niri IPC!");
        }

        Self { socket }
    }

    pub fn start_listener_thread<R: Runtime>(&mut self, app_handle: &AppHandle<R>) -> Option<()> {
        // Don't try if opening the main socket failed
        if self.socket.is_none() {
            return None;
        }

        // We need to take the current socket and create a new one so we can have
        // both open at once. Otherwise, niri closes the new one immediately.
        let socket = self.socket.take()?;

        // Open a new socket so we can read events while making requests.
        // Technically, this isn't required, since Niri sends all the required
        // information on socket open. However, since we handle niri IPC in the frontend,
        // it's nice to be able to query windows instead of needing to track them in Rust.
        let new_socket = Socket::connect();
        self.socket = new_socket
            .map_err(|e| {
                eprintln!("Failed to connect to Niri IPC: {}", e);
            })
            .ok();

        let app_handle = app_handle.clone();
        thread::spawn(move || {
            let mut event_reader = socket.read_events();
            loop {
                match event_reader() {
                    Ok(event) => {
                        app_handle
                            .emit("niri_event", event)
                            .expect("Failed to emit niri event");
                    }
                    Err(e) => {
                        eprintln!("Niri socket error: {:?}.", e);
                        break;
                    }
                }
            }
        });
        Some(())
    }
}

// Thank you, niri-ipc, for making Request/Response serde-compatible!
#[tauri::command]
async fn niri_request(
    payload: Request,
    handler: State<'_, Mutex<BarHandler>>,
) -> Result<Response, ()> {
    if let Some(socket) = &mut handler.lock().unwrap().socket {
        let response = socket
            .send(payload)
            .map_err(|e| {
                eprintln!("Failed to send niri message: {}", e);
                ()
            })?
            .map_err(|e| {
                eprintln!("Niri returned an error: {}", e);
                ()
            })?;
        Ok(response)
    } else {
        Err(())
    }
}

#[tauri::command]
fn debug_log(msg: String) {
    println!("{}", msg);
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("bar")
        .invoke_handler(tauri::generate_handler![debug_log, niri_request])
        .setup(|app, _plugin_api| {
            let mut handler = BarHandler::new();
            handler
                .start_listener_thread(app)
                .expect("Failed to start Niri event listener thread");

            app.manage(Mutex::new(handler));

            Ok(())
        })
        .build()
}
