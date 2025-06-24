use serde::Serialize;
use ts_rs::TS;
use zbus::{Result, proxy};
use zvariant::OwnedValue;

#[derive(Debug, Copy, Clone, Serialize, OwnedValue, TS)]
#[repr(u32)]
#[ts(export_to="../../bindings/UpowerProperties.ts")]
pub enum BatteryState {
    Unknown = 0,
    Charging = 1,
    Discharging = 2,
    Empty = 3,
    FullyCharged = 4,
    PendingCharge = 5,
    PendingDischarge = 6,
}

impl From<u32> for BatteryState {
    fn from(value: u32) -> Self {
        match value {
            0 => BatteryState::Unknown,
            1 => BatteryState::Charging,
            2 => BatteryState::Discharging,
            3 => BatteryState::Empty,
            4 => BatteryState::FullyCharged,
            5 => BatteryState::PendingCharge,
            6 => BatteryState::PendingDischarge,
            _ => BatteryState::Unknown, // Default case
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, OwnedValue, TS)]
#[repr(u32)]
#[ts(export_to="../../bindings/UpowerProperties.ts")]
pub enum BatteryType {
    Unknown = 0,
    LinePower = 1,
    Battery = 2,
    Ups = 3,
    Monitor = 4,
    Mouse = 5,
    Keyboard = 6,
    Pda = 7,
    Phone = 8,
}

impl From<u32> for BatteryType {
    fn from(value: u32) -> Self {
        match value {
            0 => BatteryType::Unknown,
            1 => BatteryType::LinePower,
            2 => BatteryType::Battery,
            3 => BatteryType::Ups,
            4 => BatteryType::Monitor,
            5 => BatteryType::Mouse,
            6 => BatteryType::Keyboard,
            7 => BatteryType::Pda,
            8 => BatteryType::Phone,
            _ => BatteryType::Unknown, // Default case
        }
    }
}

#[derive(Debug, Copy, Clone, OwnedValue)]
#[repr(u32)]
pub enum BatteryLevel {
    Unknown = 0,
    None = 1,
    Low = 3,
    Critical = 4,
    Normal = 6,
    High = 7,
    Full = 8,
}

#[proxy(
    interface = "org.freedesktop.UPower.Device",
    default_service = "org.freedesktop.UPower",
    assume_defaults = false
)]
pub trait Device {
    #[zbus(property)]
    fn battery_level(&self) -> Result<BatteryLevel>;

    #[zbus(property)]
    fn capacity(&self) -> Result<f64>;

    #[zbus(property)]
    fn energy(&self) -> Result<f64>;

    #[zbus(property)]
    fn energy_empty(&self) -> Result<f64>;

    #[zbus(property)]
    fn energy_full(&self) -> Result<f64>;

    #[zbus(property)]
    fn energy_full_design(&self) -> Result<f64>;

    #[zbus(property)]
    fn has_history(&self) -> Result<bool>;

    #[zbus(property)]
    fn has_statistics(&self) -> Result<bool>;

    #[zbus(property)]
    fn icon_name(&self) -> Result<String>;

    #[zbus(property)]
    fn is_present(&self) -> Result<bool>;

    #[zbus(property)]
    fn is_rechargeable(&self) -> Result<bool>;

    #[zbus(property)]
    fn luminosity(&self) -> Result<f64>;

    #[zbus(property)]
    fn model(&self) -> Result<String>;

    #[zbus(property)]
    fn native_path(&self) -> Result<String>;

    #[zbus(property)]
    fn online(&self) -> Result<bool>;

    #[zbus(property)]
    fn percentage(&self) -> Result<f64>;

    #[zbus(property)]
    fn power_supply(&self) -> Result<bool>;

    fn refresh(&self) -> Result<()>;

    #[zbus(property)]
    fn serial(&self) -> Result<String>;

    #[zbus(property)]
    fn state(&self) -> Result<BatteryState>;

    #[zbus(property)]
    fn temperature(&self) -> Result<f64>;

    #[zbus(property, name = "Type")]
    fn type_(&self) -> Result<BatteryType>;

    #[zbus(property)]
    fn vendor(&self) -> Result<String>;

    #[zbus(property)]
    fn voltage(&self) -> Result<f64>;
}

#[proxy(interface = "org.freedesktop.UPower", assume_defaults = true)]
pub trait UPower {
    /// EnumerateDevices method
    fn enumerate_devices(&self) -> Result<Vec<zvariant::OwnedObjectPath>>;

    /// GetCriticalAction method
    fn get_critical_action(&self) -> Result<String>;

    /// Get the object to the "display device", a composite device that represents the status icon
    /// to show in desktop environments. You can also access the object directly as its path is
    /// guaranteed to be /org/freedesktop/UPower/devices/DisplayDevice.
    #[zbus(object = "Device")]
    fn get_display_device(&self);

    /// DeviceAdded signal
    #[zbus(signal)]
    fn device_added(&self, device: zvariant::ObjectPath<'_>) -> Result<()>;

    /// DeviceRemoved signal
    #[zbus(signal)]
    fn device_removed(&self, device: zvariant::ObjectPath<'_>) -> Result<()>;

    /// DaemonVersion property
    #[zbus(property)]
    fn daemon_version(&self) -> Result<String>;

    /// LidIsClosed property
    #[zbus(property)]
    fn lid_is_closed(&self) -> Result<bool>;

    /// LidIsPresent property
    #[zbus(property)]
    fn lid_is_present(&self) -> Result<bool>;

    /// OnBattery property
    #[zbus(property)]
    fn on_battery(&self) -> Result<bool>;
}