use crate::{modules::{BarPosition, BarWidget, Module}, App};
use gtk4::prelude::*;

#[derive(Debug)]
pub struct SystemTray {}

impl SystemTray {
    pub fn new() -> Self {
        SystemTray {}
    }
}

impl Module for SystemTray {
    fn get_widget(&self, _app: &App) -> BarWidget {
        let tray = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::End)
            .css_classes(["system-tray"])
            .build();
        // TODO
        BarWidget {
            position: BarPosition::TopStart,
            widget: tray.upcast()
        }
    }
}