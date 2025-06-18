use crate::{modules::{BarPosition, BarWidget, Module}, App};
use gtk4::prelude::*;

#[derive(Debug)]
pub struct Clock {
}

impl Clock {
    pub fn new() -> Self {
        Clock {}
    }
}

impl Module for Clock {
    fn get_widget(&self, _app: &App) -> BarWidget {
        let widget = gtk4::Label::new(Some(""));
        widget.set_css_classes(&["clock"]);
        BarWidget {
            position: BarPosition::TopEnd,
            widget: widget.upcast()
        }
    }

    fn update_widget(&self, _app: &App, widget: &gtk4::Widget) {
        // Update the clock widget in the format "MM/DD HH:MM:SS PM/AM"
        let now = chrono::Local::now();
        let time_str = now.format("%m/%d %I:%M:%S %p").to_string();
        widget
            .downcast_ref::<gtk4::Label>()
            .expect("Widget should be a Label")
            .set_label(&time_str);
    }

    fn on_click(&self, _app: &App) {
        // TODO: Calendar popup
    }
}