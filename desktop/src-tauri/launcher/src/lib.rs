use std::sync::Mutex;

use tauri::{
    Manager, Runtime, State,
    plugin::{Builder, TauriPlugin},
};

use crate::rink::RinkResult;

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

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("launcher")
        .invoke_handler(tauri::generate_handler![rink_query, symbols_query])
        .setup(|app, _plugin_api| {
            app.manage(Mutex::new(LauncherState {
                rink_ctx: rink::create_context(),
                symbols: symbols::load_symbols(),
            }));

            Ok(())
        })
        .build()
}
