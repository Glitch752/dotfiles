use std::sync::Arc;

use std::io::Result;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri::async_runtime::Mutex;
use ts_rs::TS;
use zbus::export::ordered_stream::OrderedStreamExt;
use zbus::fdo::PropertiesProxy;
use zbus::{
    Connection,
    names::InterfaceName
};

use crate::networkmanager::dbus::{DBUS_BUS, DBUS_INTERFACE, DBUS_PATH};

mod dbus;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to="../../bindings/NetworkManagerState.ts")]
pub struct NetworkManagerState {
    pub state: ClientState,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to="../../bindings/NetworkManagerState.ts")]
pub enum ClientState {
    WiredConnected,
    WifiConnected,
    CellularConnected,
    VpnConnected,
    WifiDisconnected,
    Offline,
    Unknown,
}

pub struct NetworkManagerHandler {
    pub state: Arc<Mutex<NetworkManagerState>>,
}

impl NetworkManagerHandler {
    pub fn start_networkmanager_events<R: Runtime>(&self, app_handle: &AppHandle<R>) {
        let state = self.state.clone();
        let app_handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            match Self::start_handler(state, app_handle).await {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Failed to start NetworkManager handler: {}", e);
                }
            }
        });
    }

    async fn start_handler<R: Runtime>(
        state: Arc<Mutex<NetworkManagerState>>,
        app_handle: AppHandle<R>,
    ) -> Result<()> {
        let dbus_connection = Connection::system().await?;
        let interface_name = InterfaceName::from_static_str(DBUS_INTERFACE)?;
        let props_proxy = PropertiesProxy::builder(&dbus_connection)
            .destination(DBUS_BUS)?
            .path(DBUS_PATH)?
            .build()
            .await?;
        
        let proxy = match NetworkManagerDbusProxy::new(&dbus_connection).await {
            Ok(proxy) => proxy,
            Err(e) => {
                eprintln!("Failed to create NetworkManagerDbusProxy: {}", e);
                return;
            }
        };
        let mut primary_connection = match proxy.primary_connection().await {
            Ok(val) => val,
            Err(_) => "/".to_string(),
        };
        let mut primary_connection_type = match proxy.primary_connection_type().await {
            Ok(val) => val.to_string(),
            Err(_) => "".to_string(),
        };
        let mut wireless_enabled = match proxy.wireless_enabled().await {
            Ok(val) => val,
            Err(_) => false,
        };
        {
            let mut state_lock = state.lock().await;
            state_lock.state = determine_state(
                &primary_connection,
                &primary_connection_type,
                wireless_enabled,
            );
        }
        app_handle.manage(state.clone());
        let mut stream = match props_proxy.receive_properties_changed().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to receive properties changed: {}", e);
                return;
            }
        };
        let mut stream = stream.into_stream();
        while let Some(change) = stream.next().await {
            let args = match change.args() {
                Ok(a) => a,
                Err(_) => continue,
            };
            if args.interface_name != interface_name {
                continue;
            }
            let changed_props = &args.changed_properties;
            let mut relevant_prop_changed = false;
            if changed_props.contains_key("PrimaryConnection") {
                primary_connection = match proxy.primary_connection().await {
                    Ok(val) => val,
                    Err(_) => primary_connection.clone(),
                };
                relevant_prop_changed = true;
            }
            if changed_props.contains_key("PrimaryConnectionType") {
                primary_connection_type = match proxy.primary_connection_type().await {
                    Ok(val) => val.to_string(),
                    Err(_) => primary_connection_type.clone(),
                };
                relevant_prop_changed = true;
            }
            if changed_props.contains_key("WirelessEnabled") {
                wireless_enabled = match proxy.wireless_enabled().await {
                    Ok(val) => val,
                    Err(_) => wireless_enabled,
                };
                relevant_prop_changed = true;
            }
            if relevant_prop_changed {
                let mut state_lock = state.lock().await;
                state_lock.state = determine_state(
                    &primary_connection,
                    &primary_connection_type,
                    wireless_enabled,
                );
                let _ = app_handle.emit("networkmanager_state_changed", state_lock.clone());
            }
        }
        Ok(())
    }
}

pub async fn create_networkmanager_handler() -> Arc<NetworkManagerHandler> {
    let state = Arc::new(Mutex::new(NetworkManagerState {
        state: ClientState::Unknown,
    }));
    Arc::new(NetworkManagerHandler { state })
}

fn determine_state(
    primary_connection: &str,
    primary_connection_type: &str,
    wireless_enabled: bool,
) -> ClientState {
    if primary_connection == "/" {
        if wireless_enabled {
            ClientState::WifiDisconnected
        } else {
            ClientState::Offline
        }
    } else {
        match primary_connection_type {
            "802-3-ethernet" | "adsl" | "pppoe" => ClientState::WiredConnected,
            "802-11-olpc-mesh" | "802-11-wireless" | "wifi-p2p" => ClientState::WifiConnected,
            "cdma" | "gsm" | "wimax" => ClientState::CellularConnected,
            "vpn" | "wireguard" => ClientState::VpnConnected,
            _ => ClientState::Unknown,
        }
    }
}

#[tauri::command]
pub async fn get_networkmanager_state(state: State<'_, Mutex<NetworkManagerState>>) -> Result<NetworkManagerState, ()> {
    Ok(state.lock().await.clone())
}