use crate::{modules::{BarPosition, BarWidget, Module}, App};
use gtk4::prelude::*;

#[derive(Debug)]
pub struct Launcher {}

impl Launcher {
    pub fn new() -> Self {
        Launcher {}
    }
}

impl Launcher {
    fn open(&self, app: &App) {
        let launcher_widget = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .css_classes(["launcher"])
            .width_request(500)
            .height_request(800)
            .build();

        let search_box = gtk4::Entry::builder()
            .placeholder_text("Search...")
            .css_classes(["search-box"])
            .build();

        launcher_widget.append(&search_box);

        app.popouts.borrow_mut().open_popout::<Self>("launcher", launcher_widget.upcast(), true);
    }
}

impl Module for Launcher {
    fn get_widget(&self, _app: &App) -> BarWidget {
        let arch_logo = gtk4::Image::from_icon_name("archlinux-logo-symbolic");
        arch_logo.add_css_class("arch-icon");
        BarWidget {
            position: BarPosition::TopStart,
            widget: arch_logo.upcast()
        }
    }

    fn on_click(&self, app: &App) {
        self.open(app);
    }

    fn handle_command(&self, app: &App, command: &str) -> Option<String> {
        if command == "launcher" {
            self.open(app);
            Some("Launcher opened".to_string())
        } else {
            None
        }
    }
}