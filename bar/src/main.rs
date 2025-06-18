use std::{cell::RefCell, rc::Rc, sync::{mpsc, Arc, OnceLock}, time::Duration};

use gio::prelude::*;

use tokio::runtime::Runtime;

use crate::{bar::MonitorBars, ipc::Ipc, modules::Modules, popouts::Popouts};

pub mod bar;
mod styles;
mod popouts;
mod modules;
mod ipc;

const UPDATE_RATE: Duration = Duration::from_millis(1000);

/// Intended to be immutabe (except for internal mutable state in RefCells and the like)
#[derive(Debug)]
pub struct App {
    bars: Rc<RefCell<Vec<MonitorBars>>>,
    modules: Rc<RefCell<Modules>>,
    popouts: Rc<RefCell<Popouts>>,
    ipc: Ipc
}

impl App {
    pub fn new() -> Self {
        App {
            bars: Rc::new(RefCell::new(Vec::new())),
            modules: Rc::new(RefCell::new(Modules::new())),
            popouts: Rc::new(RefCell::new(Popouts::new())),
            ipc: Ipc::new()
        }
    }

    pub fn run(self) {
        let app = Rc::new(self);
        let app2 = app.clone();
        app.popouts.borrow_mut().app = Some(app.clone());

        let application = gtk4::Application::new(
            Some("dev.glitch752.bar"),
            Default::default()
        );
        let application2 = application.clone();

        app.ipc.start(app.clone());

        let (activated_tx, activated_rx) = mpsc::channel();

        application.connect_activate(move |app| {
            styles::load_css(Rc::new(app.clone()));
            activated_tx.send(app.hold()).unwrap();
        });

        glib::spawn_future_local(async move {
            let hold = activated_rx.recv().unwrap();

            let app = app2.clone();
            {
                // Scope to drop borrow on bars before updating
                let mut bars = app.bars.borrow_mut();

                // TODO: Support multiple monitors
                let bar = MonitorBars::new(&application2, app.clone());
                bars.push(bar);
            }

            // Update loop
            let app_clone = app.clone();
            glib::timeout_add_local(UPDATE_RATE, move || {
                app_clone.update();
                glib::ControlFlow::Continue
            });

            app.update();

            drop(hold);
        });

        application.run();
    }

    fn queue_begin_animation(self: Rc<Self>) {
        glib::idle_add_local(move || {
            self.update();
            glib::ControlFlow::Continue
        });
    }

    fn update(&self) {
        let modules = self.modules.borrow();
        modules.update_modules(&self);
        for bar in self.bars.borrow_mut().iter_mut() {
            bar.update(&self);
        }
    }

    pub fn tokio_runtime() -> Arc<Runtime> {
        static RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();
        RUNTIME.get_or_init(|| Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime")
        )).clone()
    }
    pub fn spawn<F: Future<Output = ()> + Send + 'static>(future: F) {
        Self::tokio_runtime().spawn(future);
    }
    pub fn block_on<F: Future<Output = ()> + Send + 'static>(future: F) {
        Self::tokio_runtime().block_on(future);
    }
}

fn main() {
    // If there are any arguments, we assume they are for the IPC server
    if std::env::args().len() > 1 {
        App::block_on(async {
            let args: Vec<String> = std::env::args().collect();
            let ipc = Ipc::new();
            let response = ipc.send(args[1..].join(" ")).await.expect("Failed to send IPC message");

            println!("{}", response);
        });
        return;
    }

    let app = App::new();
    app.run();
}