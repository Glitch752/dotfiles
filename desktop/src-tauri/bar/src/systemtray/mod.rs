use std::collections::HashMap;

use system_tray::{client::Client, item::StatusNotifierItem};
use tauri::{AppHandle, Manager, Runtime, State};

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

pub enum SystrayItemStatus {
    Unknown,
    Passive,
    Active,
    NeedsAttention,
}

struct SystemTrayItem {
    id: String,
    title: Option<String>,

    /// Describes the status of this item or of the associated application.
    ///
    /// The allowed values for the Status property are:
    /// - Passive: The item doesn't convey important information to the user, it can be considered an "idle" status and is likely that visualizations will chose to hide it.
    /// - Active: The item is active, is more important that the item will be shown in some way to the user.
    /// - `NeedsAttention`: The item carries really important information for the user, such as battery charge running out and is wants to incentive the direct user intervention.
    ///   Visualizations should emphasize in some way the items with `NeedsAttention` status.
    status: SystrayItemStatus,
}

#[tauri::command]
fn get_system_tray_items(
    state: State<'_, SystemTrayState>,
) -> HashMap<String, SystemTrayItem> {
    let items = state.client.items();
    let items = items.lock().unwrap();

    let mut result = HashMap::new();
    for (id, item) in items.iter() {
        result.insert(id.clone(), SystemTrayItem {
            id: item.0.id.clone(),
            title: item.0.title.clone(),
            status: SystrayItemStatus::Unknown
        });
    }

    return result;
}