use std::{cell::RefCell, rc::Rc, sync::mpsc};

use gio::prelude::*;
use gtk4::prelude::*;

use crate::bar::MonitorBars;

mod bar;
mod styles;

#[derive(Debug)]
pub struct Modules {

}

/// Intended to be immutabe (except for internal mutable state in RefCells and the like)
#[derive(Debug)]
pub struct App {
    bars: Rc<RefCell<Vec<MonitorBars>>>,
    modules: Rc<RefCell<Modules>>
}

impl App {
    pub fn new() -> Self {
        App {
            bars: Rc::new(RefCell::new(Vec::new())),
            modules: Rc::new(RefCell::new(Modules {}))
        }
    }

    pub fn run(self) {
        let app = Rc::new(self);
        let app2 = app.clone();

        let application = gtk4::Application::new(
            Some("dev.glitch752.bar"),
            Default::default()
        );
        let application2 = application.clone();

        let (activated_tx, activated_rx) = mpsc::channel();

        application.connect_activate(move |app| {
            styles::load_css(Rc::new(app.clone()));
            activated_tx.send(app.hold()).unwrap();
        });

        glib::spawn_future_local(async move {
            let hold = activated_rx.recv().unwrap();

            let app = app2.clone();
            let mut bars = app.bars.borrow_mut();

            // TODO: Support multiple monitors
            let bar = MonitorBars::new(&application2, app.clone());
            bars.push(bar);

            drop(hold);
        });

        application.run();
    }
}

fn main() {
    let app = App::new();
    app.run();
}