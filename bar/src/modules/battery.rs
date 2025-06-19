use std::process::Command;

use crate::{modules::{BarPosition, BarWidget, Module}, App};
use gtk4::prelude::*;

#[derive(Debug)]
pub struct Battery {
}

impl Battery {
    pub fn new() -> Self {
        Battery {}
    }
}

impl Module for Battery {
    fn get_widget(&self, _app: &App) -> BarWidget {
        // TODO: Icon? idk
        let widget = gtk4::Label::new(Some(""));
        widget.set_css_classes(&["battery"]);

        // let battery_icon = gtk4::Image::from_icon_name("battery-full-symbolic");
        // battery_icon.add_css_class("battery-icon");
        // horiz_end.append(&battery_icon);
        // let battery_label = gtk4::Label::new(Some("100%"));
        // battery_label.add_css_class("battery-percentage");
        // horiz_end.append(&battery_label);

        BarWidget {
            position: BarPosition::TopEnd,
            widget: widget.upcast()
        }
    }

    fn update_widget(&self, _app: &App, widget: &gtk4::Widget) {
        // TODO: Maybe there's a upower crate that could simplify this or make it more robust?
        // Update the battery widget
        let batt = Command::new("sh")
                .arg("-c")
                .arg("upower -i /org/freedesktop/UPower/devices/battery_BAT1 | grep \"percentage:\" | awk '{print $2}'")
                .output()
                .expect("failed to execute process");
        let batt_string = String::from_utf8(batt.stdout).expect("Failed to parse battery UTF-8");
        widget
            .downcast_ref::<gtk4::Label>()
            .expect("Widget should be a Label")
            .set_label(&batt_string.trim());
    }

    fn on_click(&self, _app: &App) {
        // TODO: Power profiles? I'm not sure.
    }
}