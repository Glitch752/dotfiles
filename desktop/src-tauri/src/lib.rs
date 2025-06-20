use gtk::{cairo, prelude::*};
use gtk_layer_shell::{LayerShell, Layer, Edge};
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub static BAR_THICKNESS: i32 = 28;
pub static NON_BAR_BORDER_THICKNESS: i32 = 8;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.hide().unwrap();

            let gtk_window = gtk::ApplicationWindow::new(
                &main_window.gtk_window().unwrap().application().unwrap(),
            );

            // To prevent the window from being black initially.
            gtk_window.set_app_paintable(true);

            let vbox = main_window.default_vbox().unwrap();
            main_window.gtk_window().unwrap().remove(&vbox);
            gtk_window.add(&vbox);

            // Doesn't throw errors.
            gtk_window.init_layer_shell();

            // Just works.
            gtk_window.set_layer(Layer::Top);
            gtk_window.set_anchor(Edge::Top, true);
            gtk_window.set_anchor(Edge::Bottom, true);
            gtk_window.set_anchor(Edge::Left, true);
            gtk_window.set_anchor(Edge::Right, true);

            // TODO: Actual proper update logic
            let size = (1000, 1000); 
            let rects = vec![
                cairo::RectangleInt::new(0, 0, size.0, BAR_THICKNESS),
                cairo::RectangleInt::new(0, 0, BAR_THICKNESS, size.1),
                cairo::RectangleInt::new(size.0 - NON_BAR_BORDER_THICKNESS, 0, NON_BAR_BORDER_THICKNESS, size.1),
                cairo::RectangleInt::new(0, size.1 - NON_BAR_BORDER_THICKNESS, size.0, NON_BAR_BORDER_THICKNESS)
            ];
            gtk_window.input_shape_combine_region(Some(&cairo::Region::create_rectangles(rects.as_slice())));

            gtk_window.show_all();
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap();
}
