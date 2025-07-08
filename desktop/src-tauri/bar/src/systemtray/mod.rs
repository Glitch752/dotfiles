use std::{collections::HashMap, ops::Deref, sync::Mutex};

use system_tray::client::{self, Client, UpdateEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::{systemtray::types::{SystemTrayItem, SystemTrayItems, SystrayIcon, SystrayMenu, SystrayTooltip}, BarHandler};

mod types;
// mod debouncer;

pub struct SystemTrayState {
    client: Client,
    current_items: Mutex<SystemTrayItems>
}

impl BarHandler {
    pub async fn start_system_tray_events<R: Runtime>(&mut self, app_handle: &AppHandle<R>) {
        let client = Client::new().await.expect("Failed to create system tray client");

        // Most of the time, this doesn't have initial items, but it seems like it sometimes does?
        let tray_items = client.items();
        let tray_items = tray_items.lock().unwrap();

        let mut items = HashMap::new();
        for (id, item) in tray_items.iter() {
            let (item, menu) = item;
            items.insert(id.clone(), SystemTrayItem::new(item, menu));
        }
        
        let state: SystemTrayState = SystemTrayState { client, current_items: Mutex::new(SystemTrayItems(items)) };

        let mut event_stream = state.client.subscribe();
        app_handle.manage(state);

        let app_handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            while let Ok(event) = event_stream.recv().await {
                let state = app_handle.state::<SystemTrayState>();
                let items = &mut state.current_items.lock().expect("Failed to lock system tray state");
                match event {
                    client::Event::Add(id, item) => {
                        items.insert(id, SystemTrayItem::new(&item, &None));
                    }
                    client::Event::Remove(id) => {
                        items.remove(&id);
                    }
                    client::Event::Update(id, event) => {
                        let Some(item) = items.get_mut(&id) else { return };
                        let theme = if let SystrayIcon::FreedesktopIcon { theme, .. } = &item.icon {
                            Some(theme.clone())
                        } else {
                            None
                        };
                        match event {
                            UpdateEvent::Tooltip(tooltip) => {
                                item.tooltip = tooltip.map(|tooltip| {
                                    SystrayTooltip::new(tooltip, theme)
                                });
                            }
                            UpdateEvent::Title(title) => {
                                item.title = title;
                            }
                            UpdateEvent::Status(status) => {
                                item.status = status.into();
                            }
                            UpdateEvent::Icon { icon_name, icon_pixmap } => {
                                item.icon = SystrayIcon::from_data_default(theme, icon_name, icon_pixmap);
                            }
                            UpdateEvent::OverlayIcon(name) => {
                                item.overlay_icon = SystrayIcon::from_data(
                                    theme.clone(),
                                    name,
                                    None
                                );
                            }
                            UpdateEvent::AttentionIcon(name) => {
                                item.attention_icon = SystrayIcon::from_data(
                                    theme.clone(),
                                    name,
                                    None
                                );
                            }
                            UpdateEvent::Menu(full_menu) => {
                                match item.menu {
                                    Some(ref mut menu) => {
                                        menu.update(full_menu);
                                    }
                                    None => {
                                        item.menu = Some(SystrayMenu::new(full_menu, None));
                                    }
                                }
                            }
                            UpdateEvent::MenuConnect(dbus_name) => {
                                match item.menu {
                                    Some(ref mut menu) => {
                                        menu.dbus_path = Some(dbus_name);
                                    }
                                    None => {
                                        item.menu = Some(SystrayMenu::partial(dbus_name));
                                    }
                                }
                            }
                            UpdateEvent::MenuDiff(diffs) => {
                                for diff in diffs {
                                    if let Some(menu) = &mut item.menu {
                                        menu.apply_diff(diff);
                                    } else {
                                        println!("Menu diff event received for tray item without menu: {:?}", diff);
                                    }
                                }
                            }
                        }
                    }
                }

                // Send the updated state to the frontend
                // TODO: Diffing or something? We should at least only update the item that changed.
                // TODO: Debounce
                app_handle.emit("update_tray_items", (*items).clone()).unwrap();
            }
        });
    }
}

#[tauri::command]
pub async fn get_systray_items(
    state: State<'_, SystemTrayState>
) -> Result<SystemTrayItems, ()> {
    let items = state.current_items.lock().expect("Failed to lock system tray state");
    Ok(items.deref().clone())
}