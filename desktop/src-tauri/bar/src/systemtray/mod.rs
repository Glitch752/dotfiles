use std::{collections::HashMap, sync::Mutex};

use serde::{Deserialize, Serialize};
use system_tray::{client::{self, Client, UpdateEvent}, item::{IconPixmap, Status, StatusNotifierItem, Tooltip}, menu::{Disposition, MenuDiff, MenuItem, MenuType, ToggleState, ToggleType, TrayMenu}};
use tauri::{AppHandle, Manager, Runtime};

use crate::BarHandler;

struct SystemTrayState {
    client: Client,
    current_items: HashMap<String, SystemTrayItem>
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
        
        let state: SystemTrayState = SystemTrayState { client, current_items: items };

        let mut event_stream = state.client.subscribe();
        app_handle.manage(Mutex::new(state));

        let app_handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            while let Ok(event) = event_stream.recv().await {
                let state = app_handle.state::<Mutex<SystemTrayState>>();
                let items = &mut state.lock().expect("Failed to lock system tray state").current_items;
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
                                    SystrayTooltip::from(tooltip, theme)
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
            }
        });
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
struct SystrayPixmap {
    width: i32,
    height: i32,
    pixels: Vec<u8>
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
enum SystrayMenuIcon {
    FreedesktopIcon {
        name: String
    },
    PNGData(Vec<u8>)
}

impl SystrayMenuIcon {
    fn from_data(icon_name: Option<String>, icon_data: Option<Vec<u8>>) -> Option<Self> {
        match (icon_name, icon_data) {
            (Some(name), _) => Some(SystrayMenuIcon::FreedesktopIcon {
                name
            }),
            (_, Some(data)) => Some(SystrayMenuIcon::PNGData(data)),
            _ => None
        }
    }
}

/// Note: The implementation does not itself handle ensuring that only one
/// item in a radio group is set to "on", or that a group does not have
/// "on" and "indeterminate" items simultaneously; maintaining this
/// policy is up to the toolkit wrappers.
#[derive(Debug, Serialize, Deserialize)]
enum SystrayToggleInfo {
    /// Item is an independent togglable item. If true, the item is toggled on.
    Checkmark(bool),
    /// Item is part of a group where only one item can be
    /// toggled at a time. If true, the item is toggled on.
    Radio(bool),
    /// Item cannot be toggled
    CannotBeToggled,
}

impl SystrayToggleInfo {
    fn new(toggle_state: ToggleState, toggle_type: ToggleType) -> Self {
        match (toggle_state, toggle_type) {
            (ToggleState::On, ToggleType::Checkmark) => SystrayToggleInfo::Checkmark(true),
            (ToggleState::Off | ToggleState::Indeterminate, ToggleType::Checkmark) => SystrayToggleInfo::Checkmark(false),
            (ToggleState::On, ToggleType::Radio) => SystrayToggleInfo::Radio(true),
            (ToggleState::Off | ToggleState::Indeterminate, ToggleType::Radio) => SystrayToggleInfo::Radio(false),
            (_, ToggleType::CannotBeToggled) => SystrayToggleInfo::CannotBeToggled,
        }
    }

    fn update_state(&mut self, new_state: ToggleState) {
        match (self, new_state) {
            (SystrayToggleInfo::Checkmark(state), ToggleState::On) => *state = true,
            (SystrayToggleInfo::Checkmark(state), ToggleState::Off | ToggleState::Indeterminate) => *state = false,
            (SystrayToggleInfo::Radio(state), ToggleState::On) => *state = true,
            (SystrayToggleInfo::Radio(state), ToggleState::Off | ToggleState::Indeterminate) => *state = false,
            _ => {}
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

impl From<Disposition> for SystrayMenuItemDisposition {
    fn from(disposition: Disposition) -> Self {
        match disposition {
            Disposition::Normal => SystrayMenuItemDisposition::Normal,
            Disposition::Informative => SystrayMenuItemDisposition::Informative,
            Disposition::Warning => SystrayMenuItemDisposition::Warning,
            Disposition::Alert => SystrayMenuItemDisposition::Alert,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum SystrayMenuItem {
    /// A separator. We just ignore all properties other than `visible` for separators.
    Separator {
        /// The identifier for this menu item. Used for activation and updates.
        id: i32,
        visible: bool
    },
    Item {
        /// The identifier for this menu item. Used for activation and updates.
        id: i32,
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

impl SystrayMenuItem {
    fn new(item: MenuItem) -> Self {
        match item.menu_type {
            MenuType::Separator => SystrayMenuItem::Separator {
                id: item.id,
                visible: item.visible
            },
            MenuType::Standard => {
                let submenu = if item.children_display == Some("submenu".to_string()) {
                    Some(item.submenu.into_iter().map(|sub_item| {
                        SystrayMenuItem::new(sub_item)
                    }).collect())
                } else { None };
                SystrayMenuItem::Item {
                    id: item.id,
                    label: item.label,
                    enabled: item.enabled,
                    visible: item.visible,
                    icon: SystrayMenuIcon::from_data(
                        item.icon_name,
                        item.icon_data
                    ),
                    shortcut: item.shortcut,
                    toggle_info: SystrayToggleInfo::new(item.toggle_state, item.toggle_type),
                    submenu,
                    disposition: item.disposition.into()
                }
            }
        }
    }

    fn apply_diff(&mut self, diff: MenuDiff) {
        for item in diff.remove {
            println!("Menu item {} removed; ignoring for now", item);
        }

        let update = diff.update;
        match self {
            SystrayMenuItem::Separator { visible, .. } => {
                if let Some(new_visible) = update.visible {
                    *visible = new_visible;
                }
            }
            SystrayMenuItem::Item { 
                label,
                enabled,
                visible,
                icon,
                toggle_info,
                disposition,
                ..
            } => {
                if let Some(new_label) = update.label {
                    *label = new_label;
                }
                if let Some(new_enabled) = update.enabled {
                    *enabled = new_enabled;
                }
                if let Some(new_visible) = update.visible {
                    *visible = new_visible;
                }
                if let Some(new_icon_name) = update.icon_name {
                    *icon = SystrayMenuIcon::from_data(
                        new_icon_name,
                        icon.as_ref().and_then(|icon| {
                            if let SystrayMenuIcon::PNGData(data) = icon {
                                Some(data.clone())
                            } else {
                                None
                            }
                        })
                    );
                }
                if let Some(new_icon_data) = update.icon_data {
                    *icon = SystrayMenuIcon::from_data(
                        icon.as_ref().and_then(|icon| {
                            if let SystrayMenuIcon::FreedesktopIcon { name, .. } = icon {
                                Some(name.clone())
                            } else {
                                None
                            }
                        }),
                        new_icon_data
                    );
                }
                if let Some(new_disposition) = update.disposition {
                    *disposition = new_disposition.into();
                }
                if let Some(new_toggle_state) = update.toggle_state {
                    toggle_info.update_state(new_toggle_state);
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SystrayMenu {
    /// The identifier for this menu item. Used for activation.
    id: u32,
    /// Used for activation.
    dbus_path: Option<String>,
    items: Vec<SystrayMenuItem>
}

impl SystrayMenu {
    fn partial(dbus_path: String) -> Self {
        Self {
            id: 0,
            dbus_path: Some(dbus_path),
            items: Vec::new()
        }
    }

    fn update(&mut self, tray_menu: TrayMenu) {
        self.id = tray_menu.id;
        self.items = tray_menu.submenus.into_iter().map(|item| {
            SystrayMenuItem::new(item)
        }).collect();
    }

    fn new(tray_menu: TrayMenu, dbus_path: Option<String>) -> Self {
        Self {
            id: tray_menu.id,
            dbus_path,
            items: tray_menu.submenus.into_iter().map(|item| {
                SystrayMenuItem::new(item)
            }).collect(),
        }
    }

    fn apply_diff(&mut self, diff: MenuDiff) {
        // This could be optimized, but meh.
        // BFS to find the correct menu item
        let mut queue = self.items.iter_mut().collect::<Vec<_>>();
        while let Some(item) = queue.pop() {
            match item {
                SystrayMenuItem::Item { id, .. } => {
                    if *id == diff.id {
                        item.apply_diff(diff);
                        return;
                    }

                    // If this is a submenu, we need to check its items too
                    if let SystrayMenuItem::Item { submenu: Some(items), .. } = item {
                        queue.extend(items.iter_mut());
                    }
                }
                SystrayMenuItem::Separator { id, .. } => {
                    if *id == diff.id {
                        item.apply_diff(diff);
                        return;
                    }
                }
            }
        }

        // If we reach here, the item was not found
        println!("Menu item with id {} not found in menu; ignoring diff", diff.id);
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

impl SystemTrayItem {
    fn new(item: &StatusNotifierItem, menu: &Option<TrayMenu>) -> Self {
        let theme = item.icon_theme_path.clone();
        SystemTrayItem {
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
                SystrayMenu::new(menu, item.menu.clone())
            })
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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