use std::rc::Rc;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use border::BorderWidget;
use crate::App;

mod border;

#[derive(Debug)]
pub struct MonitorBars {
    app: Rc<App>,
    /// Sadly, the only way I found to have multiple exclusive zones is to have multiple windows.
    exclusive_zones: Vec<gtk4::ApplicationWindow>,
    bar: Rc<gtk4::ApplicationWindow>,
    border_widget: BorderWidget
}

pub static BAR_THICKNESS: i32 = 28;
pub static NON_BAR_BORDER_THICKNESS: i32 = 8;

impl MonitorBars {
    pub fn new(
        application: &gtk4::Application,
        app: Rc<App>
    ) -> Self {
        let bar: Rc<gtk4::ApplicationWindow> = Rc::new(gtk4::ApplicationWindow::new(application));

        bar.init_layer_shell();
        bar.set_layer(Layer::Top);

        bar.set_anchor(Edge::Left, true);
        bar.set_anchor(Edge::Right, true);
        bar.set_anchor(Edge::Top, true);
        bar.set_anchor(Edge::Bottom, true);

        let mut bars = MonitorBars {
            app,
            exclusive_zones: vec![],
            border_widget: BorderWidget::new(bar.clone()),
            bar
        };

        bars.init_widgets();

        // Add exclusive zones for each edge
        bars.add_exclusive_zone(Edge::Top, BAR_THICKNESS);
        bars.add_exclusive_zone(Edge::Left, BAR_THICKNESS);
        bars.add_exclusive_zone(Edge::Right, NON_BAR_BORDER_THICKNESS);
        bars.add_exclusive_zone(Edge::Bottom, NON_BAR_BORDER_THICKNESS);

        bars
    }

    fn add_exclusive_zone(
        &mut self,
        edge: Edge,
        thickness: i32
    ) {
        let window = gtk4::ApplicationWindow::new(self.bar.application().as_ref().unwrap());
        window.init_layer_shell();
        
        window.set_anchor(Edge::Bottom, edge != Edge::Top);
        window.set_anchor(Edge::Top, edge != Edge::Bottom);
        window.set_anchor(Edge::Left, edge != Edge::Right);
        window.set_anchor(Edge::Right, edge != Edge::Left);
        
        window.set_layer(Layer::Top);
        
        window.set_exclusive_zone(thickness);
        
        window.show(); // "show", but the window doesn't actually have content
        self.exclusive_zones.push(window);
    }

    fn init_widgets(&mut self) {
        let overlay = gtk4::Overlay::builder()
            .hexpand(true)
            .vexpand(true)
            .halign(gtk4::Align::Fill)
            .valign(gtk4::Align::Fill)
            .build();
        overlay.add_css_class("bars");
        overlay.set_child(Some(self.border_widget.clone().widget().as_ref()));
        self.bar.set_child(Some(&overlay));

        let bars = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .halign(gtk4::Align::Fill)
            .valign(gtk4::Align::Fill)
            .vexpand(true)
            .css_classes(["bars-container"])
            .build();
        overlay.add_overlay(&bars);

        let horizontal_bar = gtk4::CenterBox::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::Fill)
            .height_request(BAR_THICKNESS-2)
            .css_classes(["horizontal-bar", "bar"])
            .build();
        bars.append(&horizontal_bar);

        let vertical_bar_container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::Fill)
            .valign(gtk4::Align::Fill)
            .vexpand(true)
            .css_classes(["vertical-bar-container"])
            .build();
        bars.append(&vertical_bar_container);

        let vertical_bar = gtk4::CenterBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .valign(gtk4::Align::Fill)
            .width_request(BAR_THICKNESS-2)
            .css_classes(["vertical-bar", "bar"])
            .build();
        vertical_bar_container.append(&vertical_bar);

        self.add_module_widgets(&horizontal_bar, &vertical_bar);

        self.bar.show();

        let surface = self.bar.surface().expect("Failed to get GDK surface");
        // TODO: Base this on the window size and reset when the window resizes
        // so we can allow inputs on the right/bottom edges
        surface.set_input_region(
            &cairo::Region::create_rectangles(&[
                cairo::RectangleInt::new(0, 0, 10000, BAR_THICKNESS),
                cairo::RectangleInt::new(0, 0, BAR_THICKNESS, 10000)
            ])
        );
    }

    fn add_module_widgets(&mut self, horizontal_bar: &gtk4::CenterBox, vertical_bar: &gtk4::CenterBox) {
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

        // The first module is, of course, an Arch Linux logo
        let arch_logo = gtk4::Image::from_icon_name("archlinux-logo-symbolic");
        arch_logo.add_css_class("arch-icon");
        horiz_start.append(&arch_logo);

        // Current window title placeholder
        let label = gtk4::Label::new(Some("Current window title here TODO"));
        label.add_css_class("window-title");
        horiz_start.append(&label);

        // I haven't figured out what goes in the center yet lol; mpris?

        // Right widgets: battery, performance, date, and clock
        // let battery_icon = gtk4::Image::from_icon_name("battery-full-symbolic");
        // battery_icon.add_css_class("battery-icon");
        // horiz_end.append(&battery_icon);
        // let battery_label = gtk4::Label::new(Some("100%"));
        // battery_label.add_css_class("battery-percentage");
        // horiz_end.append(&battery_label);

        let clock = gtk4::Label::new(Some("12:34 PM"));
        clock.add_css_class("clock");
        horiz_end.append(&clock);
        
        // let vertical_label = gtk4::Label::new(Some("Vertical Module Placeholder"));
        // vertical_label.add_css_class("vertical-module-placeholder");
        // vertical_bar.append(&vertical_label);

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

        // Start: volume, microphone, and brightness todo

        // Middle: audio player

        // End section: system tray, network, bluetooth, and power
        let tray = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::End)
            .css_classes(["system-tray"])
            .build();
        vertical_end.append(&tray);
    }
}