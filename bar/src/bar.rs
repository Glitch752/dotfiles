use std::rc::Rc;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::App;

#[derive(Debug)]
pub struct MonitorBars {
    app: Rc<App>,
    top_window: Option<gtk4::ApplicationWindow>,
    right_window: Option<gtk4::ApplicationWindow>
}

impl MonitorBars {
    pub fn new(app: Rc<App>) -> Self {
        MonitorBars {
            app,
            top_window: None,
            right_window: None
        }
    }

    pub fn activate(&mut self, application: &gtk4::Application) {
        let top = gtk4::ApplicationWindow::new(application);

        top.init_layer_shell();
        top.set_layer(Layer::Top);
        // Push other windows out of the way
        top.auto_exclusive_zone_enable();

        top.set_anchor(Edge::Left, true);
        top.set_anchor(Edge::Right, true);
        top.set_anchor(Edge::Top, true);
        top.set_anchor(Edge::Bottom, false);

        let right = gtk4::ApplicationWindow::new(application);

        right.init_layer_shell();
        right.set_layer(Layer::Top);
        // Push other windows out of the way
        right.auto_exclusive_zone_enable();

        right.set_anchor(Edge::Left, false);
        right.set_anchor(Edge::Right, true);
        right.set_anchor(Edge::Top, true);
        right.set_anchor(Edge::Bottom, true);

        // Set up a widget on each
        let label = gtk4::Label::new(Some(""));
        label.set_markup("<span font_desc=\"12.0\">wow</span>");
        top.set_child(Some(&label));
        top.show();

        let label = gtk4::Label::new(Some(""));
        label.set_markup("<span font_desc=\"6.0\">wow</span>");
        right.set_child(Some(&label));
        right.show();

        self.right_window = Some(right);
        self.top_window = Some(top);
    }
}