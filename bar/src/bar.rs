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
        let bars = gtk4::Overlay::builder()
            .hexpand(true)
            .vexpand(true)
            .halign(gtk4::Align::Fill)
            .valign(gtk4::Align::Fill)
            .build();
        bars.add_css_class("bars");

        bars.add_overlay(self.border_widget.clone().widget().as_ref());

        self.bar.set_child(Some(&bars));

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
}