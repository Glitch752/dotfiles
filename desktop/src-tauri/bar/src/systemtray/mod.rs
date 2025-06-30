use system_tray::client::Client;
use tauri::{AppHandle, Manager, Runtime};

use crate::BarHandler;

struct SystemTrayState {
    client: Client
}

impl BarHandler {
    pub async fn start_system_tray_events<R: Runtime>(&mut self, app_handle: &AppHandle<R>) {
        let state = SystemTrayState {
            client: Client::new().await.expect("Failed to create system tray client")
        };

        let mut event_stream = state.client.subscribe();

        app_handle.manage(state);

        let app_handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            while let Ok(event) = event_stream.recv().await {
                println!("System tray event: {:?}", event);
            }
        });
    }
}