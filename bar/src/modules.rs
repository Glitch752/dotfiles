use std::{fmt::Debug, rc::Rc, sync::atomic::{AtomicU32, Ordering}};

use gtk4::prelude::*;

use crate::App;

mod window_title;
mod clock;
mod launcher;
mod tray;
mod battery;

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum BarPosition {
    TopStart,
    TopCenter,
    TopEnd,
    SideStart,
    SideCenter,
    SideEnd
}

#[derive(Debug, Clone)]
pub struct BarWidget {
    position: BarPosition,
    widget: gtk4::Widget
}

pub trait Module: Debug {
    fn get_widget(&self, app: &App) -> BarWidget;
    /// Guarenteed to run before `update_widget`.
    fn update(&self, _app: &App) {
        // Default implementation does nothing
    }
    /// Guarenteed to run after `update` for each widget.
    fn update_widget(&self, _app: &App, _widget: &gtk4::Widget) {
        // Default implementation does nothing
    }
    fn on_click(&self, _app: &App) {
        // Default implementation does nothing
    }
    fn handle_command(&self, _app: &App, _command: &str) -> Option<String> {
        None
    }
}

#[derive(Debug)]
pub struct ModuleEntry {
    id: u32,
    module: Box<dyn Module>,
    widgets: Vec<BarWidget>
}

#[derive(Debug)]
pub struct Modules {
    pub modules: Vec<ModuleEntry>
}

impl Modules {
    pub fn new() -> Self {
        let mut modules = Modules {
            modules: vec![]
        };

        // The first module is, of course, an Arch Linux logo that opens the launcher
        modules.add_module(launcher::Launcher::new());
        modules.add_module(window_title::WindowTitle::new());

        // Right widgets: battery, performance, date, and clock
        modules.add_module(clock::Clock::new());


        // Start: volume, microphone, and brightness todo
        // Middle: mpris
        // End section: system tray, network, bluetooth, and power
        modules.add_module(tray::SystemTray::new());

        modules
    }

    pub fn add_module<M: Module + 'static>(&mut self, module: M) {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        self.modules.push(ModuleEntry {
            id,
            module: Box::new(module),
            widgets: vec![]
        });
    }

    pub fn update_modules(&self, app: &App) {
        for ModuleEntry { module, widgets, .. } in &self.modules {
            module.update(app);
            for widget in widgets {
                module.update_widget(app, &widget.widget);
            }
        }
    }

    pub fn add_module_widgets(&mut self, app: Rc<App>, horizontal_bar: &gtk4::CenterBox, vertical_bar: &gtk4::CenterBox) {
        // Create start, center, and end sections for the horizontal bar
        let horiz_start = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::Start)
            .css_classes(["horizontal-bar-start"])
            .build();
        let horiz_center = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::Center)
            .css_classes(["horizontal-bar-center"])
            .build();
        let horiz_end = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::End)
            .css_classes(["horizontal-bar-end"])
            .build();
        horizontal_bar.set_start_widget(Some(&horiz_start));
        horizontal_bar.set_center_widget(Some(&horiz_center));
        horizontal_bar.set_end_widget(Some(&horiz_end));

        // Vertical bar modules
        let vertical_start = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .halign(gtk4::Align::Start)
            .css_classes(["vertical-bar-start"])
            .build();
        let vertical_center = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .halign(gtk4::Align::Center)
            .css_classes(["vertical-bar-center"])
            .build();
        let vertical_end = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .halign(gtk4::Align::End)
            .css_classes(["vertical-bar-end"])
            .build();

        vertical_bar.set_start_widget(Some(&vertical_start));
        vertical_bar.set_center_widget(Some(&vertical_center));
        vertical_bar.set_end_widget(Some(&vertical_end));

        // Initialize modules
        for ModuleEntry { id, module, widgets } in &mut self.modules {
            let widget = module.get_widget(&app);
            widgets.push(widget.clone());

            let wrapper_button = gtk4::Button::builder()
                .css_classes(["module-wrapper"])
                .child(&widget.widget)
                .build();
            
            let app_clone = app.clone();
            let id = *id;
            wrapper_button.connect_clicked(move |_| {
                let app_ref = app_clone.clone();
                app_ref.modules.borrow().modules.iter()
                    .find(|ModuleEntry { id: module_id, .. }| *module_id == id)
                    .map(|ModuleEntry { module, .. }| module.on_click(&app_ref));
            });

            match widget.position {
                BarPosition::TopStart => horiz_start.append(&wrapper_button),
                BarPosition::TopCenter => horiz_center.append(&wrapper_button),
                BarPosition::TopEnd => horiz_end.append(&wrapper_button),
                BarPosition::SideStart => vertical_start.append(&wrapper_button),
                BarPosition::SideCenter => vertical_center.append(&wrapper_button),
                BarPosition::SideEnd => vertical_end.append(&wrapper_button),
            }
        }
    }

    pub fn handle_command(&mut self, command: &str, app: Rc<App>) -> Option<String> {
        for ModuleEntry { module, .. } in &self.modules {
            if let Some(response) = module.handle_command(&app, command) {
                return Some(response);
            }
        }
        None
    }
}