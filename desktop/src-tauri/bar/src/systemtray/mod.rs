use std::collections::HashMap;

use system_tray::{client::Client, item::{IconPixmap, Status, StatusNotifierItem, Tooltip}};
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

impl From<Status> for SystrayItemStatus {
    fn from(status: Status) -> Self {
        match status {
            Status::Unknown => SystrayItemStatus::Unknown,
            Status::Passive => SystrayItemStatus::Passive,
            Status::Active => SystrayItemStatus::Active,
            Status::NeedsAttention => SystrayItemStatus::NeedsAttention,
        }
    }
}

struct SystrayPixmap {
    width: i32,
    height: i32,
    pixels: Vec<u8>
}

enum SystrayIcon {
    FreedesktopIcon {
        theme: String,
        name: String
    },
    Pixmaps {
        /// The icons in this pixmap icon.
        /// Applications can provide multiple icons either for animations or
        /// multiple resolutions of the same icon.
        /// See https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/Icons/
        /// Note that the spec doesn't technically forbid including multiple icons _and_ animations.
        icons: Vec<SystrayPixmap>
    }
}

impl SystrayIcon {
    fn from_data(theme_path: Option<String>, icon_name: Option<String>, pixmap: Option<Vec<IconPixmap>>) -> Option<Self> {
        match (theme_path, icon_name, pixmap) {
            // Prefer the theme icon
            (Some(theme), Some(name), _) => Some(SystrayIcon::FreedesktopIcon {
                theme,
                name
            }),
            // If we don't have a theme icon, use the pixmap data
            (None, None, Some(pixmap)) => {
                let icons = pixmap.into_iter().map(|p| SystrayPixmap {
                    width: p.width,
                    height: p.height,
                    pixels: p.pixels
                }).collect();
                Some(SystrayIcon::Pixmaps {
                    icons
                })
            },
            // If we don't have the required data, don't return the data
            _ => None
        }
    }

    fn from_data_default(theme_path: Option<String>, icon_name: Option<String>, pixmap: Option<Vec<IconPixmap>>) -> Self {
        Self::from_data(theme_path, icon_name, pixmap).unwrap_or({
            SystrayIcon::FreedesktopIcon {
                theme: "Adwaita".to_string(),
                name: "application-x-executable".to_string()
            }
        })
    }
}

struct SystrayTooltip {
    icon: SystrayIcon,
    title: String,
    description: String,
}

impl SystrayTooltip {
    fn from(tooltip: Tooltip, theme: Option<String>) -> Self {
        SystrayTooltip {
            icon: SystrayIcon::from_data_default(
                theme,
                Some(tooltip.icon_name),
                Some(tooltip.icon_data)
            ),
            title: tooltip.title,
            description: tooltip.description
        }
    }
}

/// Why is this different from normal systray icons? I don't know...
enum SystrayMenuIcon {
    FreedesktopIcon {
        name: String
    },
    PNGData(Vec<u8>)
}

/// Note: The implementation does not itself handle ensuring that only one
/// item in a radio group is set to "on", or that a group does not have
/// "on" and "indeterminate" items simultaneously; maintaining this
/// policy is up to the toolkit wrappers.
enum SystrayToggleInfo {
    /// Item is an independent togglable item. If true, the item is toggled on.
    Checkmark(bool),
    /// Item is part of a group where only one item can be
    /// toggled at a time. If true, the item is toggled on.
    Radio(bool),
    /// Item cannot be toggled
    CannotBeToggled,
}

enum SystrayMenuItemDisposition {
    /// A standard menu item
    Normal,
    /// Providing additional information to the user
    Informative,
    /// Looking at potentially harmful results
    Warning,
    /// Something bad could potentially happen
    Alert
}

enum SystrayMenuItem {
    /// A separator. We just ignore all properties other than `visible` for separators.
    Separator {
        visible: bool
    },
    Item {
        /// The identifier for this menu item. Used for activation.
        id: u32,
        /// The text of the item, except that:
        ///  - Two consecutive underscore characters "__" are displayed as a
        ///    single underscore,
        ///  - Any remaining underscore characters are not displayed at all,
        ///  - The first of those remaining underscore characters (unless it is
        ///    the last character in the string) indicates that the following
        ///    character is the access key.
        /// Why is this so complicated lol
        label: Option<String>,
        /// Whether the item can be activated or not.
        /// Disabled items should be grayed out or similar.
        enabled: bool,
        /// If the item is visible in the menu.
        visible: bool,
        /// The menu item icon.
        icon: Option<SystrayMenuIcon>,
        /// The shortcut of the item. Each array represents the key press
        /// in the list of keypresses. Each list of strings contains a list of
        /// modifiers and then the key that is used. The modifier strings
        /// allowed are: "Control", "Alt", "Shift" and "Super".
        ///
        /// - A simple shortcut like Ctrl+S is represented as:
        ///   [["Control", "S"]]
        /// - A complex shortcut like Ctrl+Q, Alt+X is represented as:
        ///   [["Control", "Q"], ["Alt", "X"]]
        shortcut: Option<Vec<Vec<String>>>,

        /// How the menu item's information should be presented.
        toggle_info: SystrayToggleInfo,

        /// The submenu on this item.
        submenu: Option<Vec<SystrayMenuItem>>,

        /// The role of this item.
        disposition: SystrayMenuItemDisposition
    }
}

struct SystrayMenu {
    /// The identifier for this menu item. Used for activation.
    id: u32,
    /// Used for activation.
    dbus_path: String,
    items: Vec<SystrayMenuItem>
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

    icon: SystrayIcon,
    overlay_icon: Option<SystrayIcon>,
    attention_icon: Option<SystrayIcon>,

    tooltip: Option<SystrayTooltip>,

    /// If this exists, we should prefer showing the menu instead of activating the top-level item.
    menu: Option<SystrayMenu>
}

pub enum SystrayActivateRequest {
    MenuItem {
        address: String,
        menu_path: String,
        submenu_id: i32,
    },
    /// The parameter(x and y) represents screen coordinates and is to be considered an hint to the item where to show eventual windows (if any).
    Primary { address: String, x: i32, y: i32 },
    /// The parameter(x and y) represents screen coordinates and is to be considered an hint to the item where to show eventual windows (if any).
    Secondary { address: String, x: i32, y: i32 },
}

#[tauri::command]
fn get_system_tray_items(
    state: State<'_, SystemTrayState>,
) -> HashMap<String, SystemTrayItem> {
    let items = state.client.items();
    let items = items.lock().unwrap();

    let mut result = HashMap::new();
    for (id, item) in items.iter() {
        let (item, menu) = item;
        let theme = item.icon_theme_path.clone();
        result.insert(id.clone(), SystemTrayItem {
            id: item.id.clone(),
            title: item.title.clone(),
            status: item.status.into(),
            icon: SystrayIcon::from_data_default(
                theme.clone(),
                item.icon_name.clone(),
                item.icon_pixmap.clone()
            ),
            // Temporary placeholder data
            overlay_icon: SystrayIcon::from_data(
                theme.clone(),
                item.overlay_icon_name.clone(),
                item.overlay_icon_pixmap.clone()
            ),  
            attention_icon: SystrayIcon::from_data(
                theme.clone(),
                item.attention_icon_name.clone(),
                item.attention_icon_pixmap.clone()
            ),
            tooltip: item.tool_tip.clone().map(|tooltip| SystrayTooltip::from(
                tooltip.clone(),
                theme.clone()
            )),
            menu: menu.clone().map(|menu| {
                SystrayMenu { id: menu.id, dbus_path: item.menu.clone().unwrap_or("".to_string()), items: vec![
                    // TODO
                ] }
            })
        });
    }

    return result;
}