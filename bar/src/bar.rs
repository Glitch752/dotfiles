use std::{cell::RefCell, rc::Rc};

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use border::BorderWidget;
use crate::App;

pub mod border;

#[derive(Debug)]
pub struct MonitorBars {
    app: Rc<App>,
    /// Sadly, the only way I found to have multiple exclusive zones is to have multiple windows.
    exclusive_zones: Vec<gtk4::ApplicationWindow>,
    bar: Rc<gtk4::ApplicationWindow>,
    border_widget: Rc<RefCell<BorderWidget>>
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

        bar.show();

        let surface = bar.surface().expect("Failed to get GDK surface");

        let mut bars = MonitorBars {
            app,
            exclusive_zones: vec![],
            border_widget: Rc::new(RefCell::new(BorderWidget::new(surface))),
            bar
        };

        bars.init_widgets();

        BorderWidget::configure_input_region_handling(bars.border_widget.clone());

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
        self.bar.set_child(Some(&overlay));
        
        overlay.set_child(Some(self.border_widget.borrow().widget().as_ref()));
        
        let bars = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .halign(gtk4::Align::Fill)
            .valign(gtk4::Align::Fill)
            .vexpand(true)
            .css_classes(["bars-container"])
            .build();
        
        overlay.add_overlay(&self.app.popouts.borrow_mut().init_container());
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

        self.app.modules.borrow_mut().add_module_widgets(self.app.clone(), &horizontal_bar, &vertical_bar);
    }


    pub fn update(&mut self, app: &App) {
        self.border_widget.borrow_mut().set_popout_positions(&app.popouts.borrow().open);
    }
}