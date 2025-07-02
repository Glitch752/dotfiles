use std::{path::PathBuf, sync::Mutex};

use tauri::{
    Manager, Runtime, State,
    plugin::{Builder, TauriPlugin},
};

use crate::{desktop_files::{DesktopFile, DesktopFiles}, rink::RinkResult};

pub use freedesktop_icon_lookup;

pub mod desktop_files;
mod rink;
mod symbols;

struct LauncherState {
    rink_ctx: rink_core::Context,
    symbols: symbols::Symbols,
}

#[tauri::command]
async fn rink_query(
    payload: String,
    state: State<'_, Mutex<LauncherState>>,
) -> Result<RinkResult, ()> {
    let mut state = state.lock().unwrap();
    Ok(rink::execute(&mut state.rink_ctx, &payload))
}

#[tauri::command]
async fn symbols_query(
    payload: String,
    state: State<'_, Mutex<LauncherState>>,
) -> Result<Vec<symbols::Symbol>, ()> {
    let mut state = state.lock().unwrap();
    Ok(symbols::execute(&mut state.symbols, &payload))
}

#[tauri::command]
async fn applications_query(
    payload: String,
    state: State<'_, DesktopFiles>,
) -> Result<Vec<DesktopFile>, ()> {
    Ok(state.fuzzy_search(payload).await.map_err(|_| ())?)
}

/// Starts the given application using a shell and disowns it.
/// This is probably super insecure or something, but whatever lol
#[tauri::command]
fn start_application(
    payload: String
) -> () {
    use std::process::Command;

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("setsid {} &", payload))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn process");
}

#[tauri::command]
async fn reload_desktop_files(
    desktop_files: State<'_, DesktopFiles>,
) -> Result<(), ()> {
    desktop_files.reload().await.map_err(|_| ())
}

#[tauri::command]
async fn resolve_icon(icon: String, theme: String, desktop_files: State<'_, DesktopFiles>) -> Result<Option<PathBuf>, ()> {
    let mut cache = desktop_files.icon_cache.lock().await;

    if cache.themes().position(|t| t == theme).is_none() {
        match cache.load(theme.clone()) {
            Ok(_) => {},
            Err(_) => return Ok(None)
        }
    }

    Ok(cache.lookup(&icon, Some(theme.as_str())))
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("launcher")
        .invoke_handler(tauri::generate_handler![
            rink_query,
            symbols_query,
            applications_query,
            start_application,
            resolve_icon,
            reload_desktop_files
        ])
        .setup(|app, _plugin_api| {
            app.manage(Mutex::new(LauncherState {
                rink_ctx: rink::create_context(),
                symbols: symbols::load_symbols(),
            }));

            app.manage(DesktopFiles::new());

            Ok(())
        })
        .build()
}
