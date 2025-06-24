use tauri::{
    plugin::{Builder, TauriPlugin}, AppHandle, Manager, Runtime
};
use zbus::{connection, Connection};

use crate::daemon::NotificationDaemon;

mod daemon;

async fn start_connection<R: Runtime>(app_handle: AppHandle<R>) -> zbus::Result<Connection> {
    connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", NotificationDaemon::new(app_handle))?
        .build()
        .await
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("notifications")
        .invoke_handler(tauri::generate_handler![])
        .setup(|app, _plugin_api| {
            let app_handle = app.app_handle().clone();

            tauri::async_runtime::spawn(async move {
                let connection = start_connection(app_handle.clone()).await;
                if let Err(e) = connection {
                    eprintln!("Failed to start DBus connection: {}", e);
                    return;
                }
                let connection = connection.unwrap();
                app_handle.manage(connection);
            });

            Ok(())
        })
        .build()
}
