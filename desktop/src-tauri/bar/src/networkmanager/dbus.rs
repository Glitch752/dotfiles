use zbus::{Result, proxy};
use zvariant::{ObjectPath, Str};

pub(super) const DBUS_BUS: &str = "org.freedesktop.NetworkManager";
pub(super) const DBUS_PATH: &str = "/org/freedesktop/NetworkManager";
pub(super) const DBUS_INTERFACE: &str = "org.freedesktop.NetworkManager";

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
pub(super) trait NetworkManagerDbus {
    #[zbus(property)]
    fn active_connections(&self) -> Result<Vec<ObjectPath>>;

    #[zbus(property)]
    fn devices(&self) -> Result<Vec<ObjectPath>>;

    #[zbus(property)]
    fn networking_enabled(&self) -> Result<bool>;

    #[zbus(property)]
    fn primary_connection(&self) -> Result<ObjectPath>;

    #[zbus(property)]
    fn primary_connection_type(&self) -> Result<Str>;

    #[zbus(property)]
    fn wireless_enabled(&self) -> Result<bool>;
}
