use std::{rc::Rc, sync::atomic::{AtomicU32, Ordering}, time::Instant};

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use border::BorderWidget;
use crate::{bar::popouts::Popouts, modules::Module, App};

pub mod border;
pub mod popouts;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarId(u32);

#[derive(Debug)]
pub struct MonitorBars {
    app: Rc<App>,
    pub id: BarId,
    /// Sadly, the only way I found to have multiple exclusive zones is to have multiple windows.
    exclusive_zones: Vec<gtk4::ApplicationWindow>,
    window: Rc<gtk4::ApplicationWindow>,
    border_widget: BorderWidget,
    last_frame_time: Instant,
    animating: bool,
    popouts: Popouts
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

        bar.set_keyboard_mode(KeyboardMode::OnDemand);

        bar.show();

        let surface = bar.surface().expect("Failed to get GDK surface");

        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        let id = BarId(NEXT_ID.fetch_add(1, Ordering::SeqCst));

        let mut bars = MonitorBars {
            app: app.clone(),
            id,
            exclusive_zones: vec![],
            border_widget: BorderWidget::new(app, id, surface),
            popouts: Popouts::new(),
            last_frame_time: Instant::now(),
            animating: false,
            window: bar
        };

        bars.init_widgets();

        // Add exclusive zones for each edge
        bars.add_exclusive_zone(Edge::Top, BAR_THICKNESS);
        bars.add_exclusive_zone(Edge::Left, BAR_THICKNESS);
        bars.add_exclusive_zone(Edge::Right, NON_BAR_BORDER_THICKNESS);
        bars.add_exclusive_zone(Edge::Bottom, NON_BAR_BORDER_THICKNESS);

        bars
    }

    pub fn open_popout<SourceModule: 'static + Module>(&mut self, id: &str, widget: gtk4::Widget, takes_keyboard_focus: bool) {
        self.popouts.open_popout::<SourceModule>(id, widget, takes_keyboard_focus);
        self.animate_until_finished();
    }

    fn add_exclusive_zone(
        &mut self,
        edge: Edge,
        thickness: i32
    ) {
        let window = gtk4::ApplicationWindow::new(self.window.application().as_ref().unwrap());
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
        self.window.set_child(Some(&overlay));
        
        overlay.set_child(Some(self.border_widget.widget()));
        
        let bars = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .halign(gtk4::Align::Fill)
            .valign(gtk4::Align::Fill)
            .vexpand(true)
            .css_classes(["bars-container"])
            .build();
        
        overlay.add_overlay(&bars);
        overlay.add_overlay(&self.popouts.init_container());

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

    fn animate_until_finished(&mut self) {
        if self.animating {
            // We're already animating, so we don't need to do anything
            return;
        }
        self.animating = true;

        self.last_frame_time = Instant::now();
        self.animate();
    }

    /// Performs animation. This is run per-bar so we can sync to the monitor's refresh rate.
    pub fn animate(&mut self) {
        if !self.animating {
            return;
        }

        let dt = self.last_frame_time.elapsed();
        println!("Delta: {:?}", dt);
        self.last_frame_time = Instant::now();

        let should_continue_animating = self.popouts.animate(dt);
        self.border_widget.set_popout_positions(&self.popouts.open);

        // Draw one last time even if we stop animating
        let app = self.app.clone();
        let id = self.id;
        // Hack? I don't even know what to think of this garbage...
        // TODO: Figure out the proper way to do this
        glib::idle_add_local_once(move || {
            app.borrow_bar_mut(id).border_widget.widget().queue_draw();
        });

        if !should_continue_animating {
            self.animating = false;
        }
    }
}