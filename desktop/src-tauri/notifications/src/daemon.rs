use serde::Serialize;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Runtime;
use ts_rs::TS;
use zbus::interface;
use zvariant::Value;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to="../../bindings/Notification.ts")]
struct Notification {
    id: u32,
    application_name: String,
    application_icon: Option<String>,
    title: String,
    body: String,
    actions: Vec<NotificationAction>,
    urgency: NotificationUrgency
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export_to="../../bindings/Notification.ts")]
struct NotificationAction {
    action_key: String,
    label: String
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export_to="../../bindings/Notification.ts")]
enum NotificationUrgency {
    Low,
    Normal,
    Critical
}

pub struct NotificationDaemon<R: Runtime> {
    notification_id: AtomicU32,
    app_handle: AppHandle<R>
}

impl<R: Runtime> NotificationDaemon<R> {
    pub fn new(app_handle: AppHandle<R>) -> Self {
        NotificationDaemon {
            notification_id: AtomicU32::new(1),
            app_handle
        }
    }

    fn get_next_id(&self) -> u32 {
        // Wraps on overflow, although we probably won't have 4,294,967,295 notifications
        self.notification_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    fn send_notification_added(&self, notification: Notification) {
        self.app_handle.emit("notification_added", notification)
            .expect("Failed to send notification_added event");
    }
    fn send_notification_removed(&self, id: u32) {
        self.app_handle.emit("notification_removed", id)
            .expect("Failed to send notification_removed event");
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl<R: Runtime> NotificationDaemon<R> {
    fn notify(
        &self,
        // The calling app's name
        app_name: &str,
        // ID of a previous notification to replace, or 0 to show a new one
        replaces_id: u32,
        // Name/path of an icon to display
        app_icon: &str,
        // Notification title
        summary: &str,
        // Main body text. Seemingly can include Pango markup?
        body: &str,
        // Pairs of (action key, label) like ["default", "Open"]
        actions: Vec<String>,
        // Optional metadata that isn't really standardized
        // Common hints:
        // - urgency | 0 = Low, 1 = Normal, 2 = Critical
        // - category | e.g. "email", "call", "im"
        // - desktop-entry | App's .desktop filename (only sometimes without .desktop suffix?)
        // - image-path | Path to image to show
        hints: HashMap<String, Value>,
        // Time in milliseconds to show notification; -1 = default and 0 = persistent
        _expire_timeout: i32,
    ) -> u32 {
        let id = self.get_next_id();

        let urgency = match hints.get("urgency") {
            Some(Value::U8(0) | Value::U16(0) | Value::U32(0)) => NotificationUrgency::Low,
            Some(Value::U8(1) | Value::U16(1) | Value::U32(1)) => NotificationUrgency::Normal,
            Some(Value::U8(2) | Value::U16(2) | Value::U32(2)) => NotificationUrgency::Critical,
            _ => NotificationUrgency::Normal, // Default to Normal if not specified
        };

        let actions = actions
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(NotificationAction {
                        action_key: chunk[0].clone(),
                        label: chunk[1].clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        let app_icon = if app_icon.is_empty() {
            None
        } else {
            Some(app_icon.to_string())
        };

        let notification = Notification {
            id,
            application_name: app_name.to_string(),
            application_icon: app_icon,
            title: summary.to_string(),
            body: body.to_string(),
            actions,
            urgency
        };

        self.send_notification_added(notification);

        if replaces_id != 0 {
            self.send_notification_removed(replaces_id);
        }

        id
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec!["body".into(), "actions".into(), "icon".into(), "persistence".into()]
    }

    fn get_server_information(&self) -> (String, String, String, String) {
        (
            "desktop-notificaations-daemon".into(),
            "Glitch752".into(),
            "1.0".into(),
            "1.2".into()
        )
    }

    fn close_notification(&self, id: u32) {
        println!("Close notification ID: {}", id);
    }
}
