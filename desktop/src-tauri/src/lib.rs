use std::{ops::Deref, sync::Mutex};

use gtk::{cairo, prelude::*};
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use ts_rs::TS;

pub use crate::ipc::Ipc;
pub mod ipc;

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
struct InputRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
struct ExclusiveRegions {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

// Safety: This is only used for GTK constructs, which check if they're used on the main thread
struct TrustMeThisWillOnlyBeUsedOnTheMainThread<T>(T);
unsafe impl<T> Send for TrustMeThisWillOnlyBeUsedOnTheMainThread<T> {}
unsafe impl<T> Sync for TrustMeThisWillOnlyBeUsedOnTheMainThread<T> {}
impl<T> Deref for TrustMeThisWillOnlyBeUsedOnTheMainThread<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct AppState {
    // Safety: Only accessed from sync commands, which are run on the main thread in Tauri
    gtk_window: TrustMeThisWillOnlyBeUsedOnTheMainThread<gtk::ApplicationWindow>,
    exclusive_zones: Vec<TrustMeThisWillOnlyBeUsedOnTheMainThread<gtk::ApplicationWindow>>,
    gtk_application: TrustMeThisWillOnlyBeUsedOnTheMainThread<gtk::Application>,
}

impl AppState {
    fn add_exclusive_zone(&mut self, edge: Edge, thickness: i32) {
        let window = gtk::ApplicationWindow::new(self.gtk_application.deref());
        window.init_layer_shell();

        window.set_anchor(Edge::Bottom, edge != Edge::Top);
        window.set_anchor(Edge::Top, edge != Edge::Bottom);
        window.set_anchor(Edge::Left, edge != Edge::Right);
        window.set_anchor(Edge::Right, edge != Edge::Left);

        window.set_layer(Layer::Top);

        window.set_exclusive_zone(thickness);

        window.show(); // "show", but the window doesn't actually have content
        self.exclusive_zones
            .push(TrustMeThisWillOnlyBeUsedOnTheMainThread(window));
    }
}

#[tauri::command]
fn set_keyboard_exclusivity(payload: bool, state: State<'_, Mutex<AppState>>) {
    state
        .lock()
        .unwrap()
        .gtk_window
        .set_keyboard_mode(if payload {
            KeyboardMode::Exclusive
        } else {
            KeyboardMode::OnDemand
        });
}

#[tauri::command]
fn set_input_shape(payload: Vec<InputRect>, state: State<'_, Mutex<AppState>>) {
    let rects = payload
        .iter()
        .map(|r| cairo::RectangleInt::new(r.x, r.y, r.width, r.height))
        .collect::<Vec<_>>();
    state
        .lock()
        .unwrap()
        .gtk_window
        .input_shape_combine_region(Some(&cairo::Region::create_rectangles(rects.as_slice())));
}

#[tauri::command]
fn devtools(payload: bool, app_handle: tauri::AppHandle) {
    let window = app_handle.get_webview_window("main").unwrap();
    if payload {
        #[cfg(debug_assertions)]
        window.open_devtools();
    } else {
        #[cfg(debug_assertions)]
        window.close_devtools();
    }
}

#[tauri::command]
fn create_exclusive_regions(payload: ExclusiveRegions, state: State<'_, Mutex<AppState>>) {
    // TODO: Exclusive region logic needs to be updated once we support multiple monitors
    let mut state = state.lock().unwrap();

    if state.exclusive_zones.len() > 0 {
        println!("Not re-creating exclusive regions");
        return;
    }

    state.add_exclusive_zone(Edge::Top, payload.top);
    state.add_exclusive_zone(Edge::Left, payload.left);
    state.add_exclusive_zone(Edge::Right, payload.right);
    state.add_exclusive_zone(Edge::Bottom, payload.bottom);
}

#[tauri::command]
fn inspect() {
    // Will run on the main thread
    gtk::Window::set_interactive_debugging(true);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            println!("Already running!");
        }))
        .plugin(bar::init())
        .plugin(launcher::init())
        .plugin(notifications::init())
        .invoke_handler(tauri::generate_handler![
            set_input_shape,
            create_exclusive_regions,
            set_keyboard_exclusivity,
            inspect,
            devtools
        ])
        .setup(|app| {
            // TODO: Support multiple windows by manually handling webview creation

            let main_window = app.get_webview_window("main").unwrap();
            // This is kind of sketchy, but it works.
            // We hide the original GTK window that Tauri makes and create our own
            // to be used for gtk-layer-shell.
            main_window.hide().unwrap();

            let tauri_gtk_window = main_window
                .gtk_window()
                .expect("Failed to get Tauri GTK window");
            let gtk_application = tauri_gtk_window
                .application()
                .expect("Failed to get Tauri GTK application");

            let gtk_window = gtk::ApplicationWindow::new(&gtk_application);

            // To prevent the window from being black initially.
            gtk_window.set_app_paintable(true);

            let vbox = main_window.default_vbox().unwrap();
            tauri_gtk_window.remove(&vbox);
            gtk_window.add(&vbox);

            // Doesn't throw errors.
            gtk_window.init_layer_shell();

            // Just works.
            gtk_window.set_layer(Layer::Top);
            gtk_window.set_anchor(Edge::Top, true);
            gtk_window.set_anchor(Edge::Bottom, true);
            gtk_window.set_anchor(Edge::Left, true);
            gtk_window.set_anchor(Edge::Right, true);

            gtk_window.show_all();

            // Before the UI starts, clear the input region so we don't eat mouse inputs immediately
            gtk_window.input_shape_combine_region(Some(&cairo::Region::create_rectangles(&[])));
            gtk_window.set_keyboard_mode(gtk_layer_shell::KeyboardMode::OnDemand);

            let ipc = Ipc::new();
            ipc.start(app.handle().clone());

            app.manage(Mutex::new(AppState {
                gtk_window: TrustMeThisWillOnlyBeUsedOnTheMainThread(gtk_window),
                gtk_application: TrustMeThisWillOnlyBeUsedOnTheMainThread(gtk_application),
                exclusive_zones: vec![],
            }));

            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap();
}
