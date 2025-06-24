mod dbus;

use dbus::UPowerProxy;
use futures::StreamExt;
use serde::Serialize;
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use ts_rs::TS;
use zbus::fdo::PropertiesProxy;
use zbus::names::InterfaceName;
use zbus::proxy::CacheProperties;
use zbus::export::ordered_stream::OrderedStreamExt;

/// See https://upower.freedesktop.org/docs/Device.html for documentation on the interface, and
/// see https://upower.freedesktop.org/docs/UPower.html for documentaation on the display device.
/// Since this only references the display device from UPower, only some properties are set:
/// Type: the type of the display device, UPS or Battery. Note that this value can change, as opposed to real devices.
/// State: the power state of the display device, such as Charging or Discharging.
/// Percentage: the amount of energy left on the device.
/// Energy: Amount of energy (measured in Wh) currently available in the power source.
/// EnergyFull: Amount of energy (measured in Wh) in the power source when it's considered full.
/// EnergyRate: Discharging/charging rate of the source, measured in Watt.
/// TimeToEmpty: Number of seconds until the power source is considered empty.
/// TimeToFull: Number of seconds until the power source is considered full.
/// IsPresent: Whether a status icon using this information should be presented. (Takes a special meaning)
/// IconName: An icon name representing the device state.
/// WarningLevel: The same as the overall WarningLevel
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to="../../bindings/UpowerProperties.ts")]
pub struct UpowerProperties {
    /// The battery power state.
    state: BatteryState,
    /// The amount of energy left in the power source expressed as a percentage between 0 and 100.
    /// Typically this is the same as (energy - energy-empty) / (energy-full - energy-empty).
    /// However, some primitive power sources are capable of only reporting percentages and in
    /// this case the energy-* properties will be unset while this property is set.
    /// 
    /// The percentage will still exist as an approximation if the device only supports coarse
    /// reporting, so we don't need to worry about that.
    percentage: f64,
    /// Amount of energy (measured in Wh) currently available in the power source.
    energy: f64,
    /// Amount of energy (measured in Wh) in the power source when it's considered full.
    energy_full: f64,
    /// Discharging/charging rate of the source, measured in Watt.
    energy_rate: f64,
    /// Number of seconds until the power source is considered empty. Set to 0 if unknown. 
    time_to_empty: i64,
    /// Number of seconds until the power source is considered full. Set to 0 if unknown. 
    time_to_full: i64,
    /// This takes a special meaning compared to real devices!
    /// Set to true if battery information should be presented to the user.
    is_present: bool,
    /// An icon name, following the Icon Naming Specification.
    /// Note that the icons might not match end-user expectations in terms of
    /// presentation relative to the amount of battery left or perceived to be left.
    /// It is recommended that front-ends use the BatteryLevel property first,
    /// if available, followed by the Percentage, to present a more realistic battery level to the user. 
    icon_name: String
}

pub async fn create_upower_proxy() -> zbus::Result<PropertiesProxy<'static>> {
    let connection = zbus::Connection::system().await?;
    let dbus = Box::pin(connection);

    let device_proxy = UPowerProxy::new(&dbus).await?;
    let display_device = device_proxy.get_display_device().await?;

    let path = display_device.inner().path();

    let proxy = PropertiesProxy::builder(&dbus)
        .destination("org.freedesktop.UPower")
        .expect("failed to set proxy destination address")
        .path(path)
        .expect("failed to set proxy path")
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    Ok(proxy)
}

use crate::upower::dbus::BatteryState;
use crate::BarHandler;

impl BarHandler {
    pub fn start_upower_events<R: Runtime>(&mut self, app_handle: &AppHandle<R>) {
        let upower_proxy = match &self.upower_proxy {
            Some(proxy) => proxy.clone(),
            None => {
                eprintln!("UPower proxy is not initialized.");
                return;
            }
        };

        let app_handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            let event_stream = upower_proxy.receive_properties_changed()
                .await
                .expect("Failed to create UPower event stream");

            let device_interface_name =
                InterfaceName::from_static_str("org.freedesktop.UPower.Device")
                    .expect("Failed to create zbus InterfaceName");

            let properties = upower_proxy.get_all(device_interface_name.clone()).await
                .expect("Failed to get properties from UPower proxy");
            
            // Set initial values
            let upower_properties = Mutex::new(UpowerProperties {
                state: properties["State"]
                    .downcast_ref::<u32>()
                    .expect("expected state: BatteryState in HashMap")
                    .into(),
                percentage: properties["Percentage"]
                    .downcast_ref::<f64>()
                    .expect("expected percentage: f64 in HashMap"),
                energy: properties["Energy"]
                    .downcast_ref::<f64>()
                    .expect("expected energy: f64 in HashMap"),
                energy_full: properties["EnergyFull"]
                    .downcast_ref::<f64>()
                    .expect("expected energy_full: f64 in HashMap"),
                energy_rate: properties["EnergyRate"]
                    .downcast_ref::<f64>()
                    .expect("expected energy_rate: f64 in HashMap"),
                time_to_empty: properties["TimeToEmpty"]
                    .downcast_ref::<i64>()
                    .expect("expected time_to_empty: i64 in HashMap"),
                time_to_full: properties["TimeToFull"]
                    .downcast_ref::<i64>()
                    .expect("expected time_to_full: i64 in HashMap"),
                is_present: properties["IsPresent"]
                    .downcast_ref::<bool>()
                    .expect("expected is_present: bool in HashMap"),
                icon_name: properties["IconName"]
                    .downcast_ref::<String>()
                    .expect("expected icon_name: String in HashMap")
                    .clone()
            });

            app_handle.manage(upower_properties);

            let mut event_stream = event_stream.into_stream();
            while let Some(event) = event_stream.next().await {
                let args = event.args().expect("Invalid signal arguments");
                if args.interface_name != device_interface_name {
                    continue;
                }

                let upower_properties = app_handle.state::<Mutex<UpowerProperties>>();
                let mut upower_properties = upower_properties.lock().await;

                for (name, changed_value) in args.changed_properties {
                    macro_rules! update_property {
                        ($field:ident, $type:ty) => {
                            if let Some(value) = changed_value.downcast_ref::<$type>().ok() {
                                upower_properties.$field = value.into();
                            }
                        };
                    }

                    match name {
                        "State" => update_property!(state, u32),
                        "Percentage" => update_property!(percentage, f64),
                        "Energy" => update_property!(energy, f64),
                        "EnergyFull" => update_property!(energy_full, f64),
                        "EnergyRate" => update_property!(energy_rate, f64),
                        "TimeToEmpty" => update_property!(time_to_empty, i64),
                        "TimeToFull" => update_property!(time_to_full, i64),
                        "IsPresent" => update_property!(is_present, bool),
                        "IconName" => update_property!(icon_name, String),
                        _ => {}
                    }
                }

                app_handle.emit("upower_properties_changed", upower_properties.clone())
                    .expect("failed to emit UPower properties changed event");
            }
        });
    }
}

#[tauri::command]
pub async fn get_upower_properties(upower_properties: State<'_, Mutex<UpowerProperties>>) -> Result<UpowerProperties, ()> {
    Ok(upower_properties.lock().await.clone())
}