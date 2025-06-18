use crate::{modules::{BarPosition, BarWidget, Module}, App};
use gtk4::prelude::*;

#[derive(Debug)]
pub struct WindowTitle {}

impl WindowTitle {
    pub fn new() -> Self {
        WindowTitle {}
    }
}

impl Module for WindowTitle {
    fn get_widget(&self, _app: &App) -> BarWidget {
        // TODO
        let label = gtk4::Label::new(Some("Current window title here TODO"));
        label.add_css_class("window-title");
        BarWidget {
            position: BarPosition::TopStart,
            widget: label.upcast()
        }
    }
}