use futures::StreamExt;
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

use crate::networkmanager::dbus::{NetworkManagerDbusProxy, DBUS_BUS, DBUS_INTERFACE, DBUS_PATH};
use crate::BarHandler;

mod dbus;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to="../../bindings/NetworkManagerState.ts")]
pub struct NetworkManagerState {
    pub status: NetworkStatus,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to="../../bindings/NetworkManagerState.ts")]
pub enum NetworkStatus {
    WiredConnected,
    WifiConnected,
    CellularConnected,
    VpnConnected,
    WifiDisconnected,
    Offline,
    Unknown,
}

impl BarHandler {
    pub fn start_networkmanager_events<R: Runtime>(&self, app_handle: &AppHandle<R>) {
        let state = Mutex::new(NetworkManagerState {
            status: NetworkStatus::Unknown,
        });
        app_handle.manage(state);

        let app_handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            match Self::start_networkmanager_handler(app_handle).await {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Failed to start NetworkManager handler: {}", e);
                }
            }
        });
    }

    async fn start_networkmanager_handler<R: Runtime>(app_handle: AppHandle<R>) -> zbus::Result<()> {
        let dbus_connection = Connection::system().await?;
        let interface_name = InterfaceName::from_static_str(DBUS_INTERFACE)?;
        let props_proxy = PropertiesProxy::builder(&dbus_connection)
            .destination(DBUS_BUS)?
            .path(DBUS_PATH)?
            .build()
            .await?;
        
        let proxy = NetworkManagerDbusProxy::new(&dbus_connection).await?;

        let mut primary_connection = proxy.primary_connection().await?;
        let mut primary_connection_type = proxy.primary_connection_type().await?;
        let mut wireless_enabled = proxy.wireless_enabled().await?;

        {
            let state = app_handle.state::<Mutex<NetworkManagerState>>();
            let mut state = state.lock().await;
            state.status = determine_state(
                &primary_connection,
                &primary_connection_type,
                wireless_enabled,
            );
        }

        let stream = props_proxy.receive_properties_changed().await?;
        let mut stream = stream.into_stream();
        
        while let Some(change) = stream.next().await {
            let args = change.args()?;
            if args.interface_name != interface_name {
                continue;
            }

            let changed_props = &args.changed_properties;
            let mut relevant_prop_changed = false;
            
            if changed_props.contains_key("PrimaryConnection") {
                primary_connection = proxy.primary_connection().await?;
                relevant_prop_changed = true;
            }
            if changed_props.contains_key("PrimaryConnectionType") {
                primary_connection_type = proxy.primary_connection_type().await?;
                relevant_prop_changed = true;
            }
            if changed_props.contains_key("WirelessEnabled") {
                wireless_enabled = proxy.wireless_enabled().await?;
                relevant_prop_changed = true;
            }

            if relevant_prop_changed {
                let state = app_handle.state::<Mutex<NetworkManagerState>>();
                let mut state = state.lock().await;
                state.status = determine_state(
                    &primary_connection,
                    &primary_connection_type,
                    wireless_enabled,
                );
                let _ = app_handle.emit("networkmanager_state_changed", state.clone());
            }
        }
        Ok(())
    }
}

fn determine_state(
    primary_connection: &str,
    primary_connection_type: &str,
    wireless_enabled: bool,
) -> NetworkStatus {
    if primary_connection == "/" {
        if wireless_enabled {
            NetworkStatus::WifiDisconnected
        } else {
            NetworkStatus::Offline
        }
    } else {
        match primary_connection_type {
            "802-3-ethernet" | "adsl" | "pppoe" => NetworkStatus::WiredConnected,
            "802-11-olpc-mesh" | "802-11-wireless" | "wifi-p2p" => NetworkStatus::WifiConnected,
            "cdma" | "gsm" | "wimax" => NetworkStatus::CellularConnected,
            "vpn" | "wireguard" => NetworkStatus::VpnConnected,
            _ => NetworkStatus::Unknown,
        }
    }
}

#[tauri::command]
pub async fn get_networkmanager_state(state: State<'_, Mutex<NetworkManagerState>>) -> Result<NetworkManagerState, ()> {
    Ok(state.lock().await.clone())
}