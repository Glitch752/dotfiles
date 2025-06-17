// TODO: Hot reload CSS

use gtk4::gdk::Display;
use notify::{recommended_watcher, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::{Path, PathBuf}, time::Instant};
use tokio::sync::mpsc;
use notify::{Event, Error, EventHandler};

struct TokioSenderHandler {
    sender: mpsc::Sender<Result<Event, Error>>,
}

impl EventHandler for TokioSenderHandler {
    fn handle_event(&mut self, event: Result<Event, Error>) {
        // We ignore send errors (e.g., if receiver is dropped)
        let _ = self.sender.try_send(event);
    }
}

pub fn load_css() {
    let css_provider = gtk4::CssProvider::new();
    let css_file = PathBuf::from("static/style.css");
    css_provider.load_from_path(css_file);

    let (tx, mut rx) = mpsc::channel(5);

    let mut watcher: RecommendedWatcher = recommended_watcher(TokioSenderHandler { sender: tx }).unwrap();
    watcher.watch(Path::new("static/style.css"), RecursiveMode::Recursive).unwrap();

    // Keep watcher alive by leaking it. This is sort of hacky, but meh.
    let _watcher_leak = Box::leak(Box::new(watcher));

    let css_provider_clone = css_provider.clone();
    glib::spawn_future_local(async move {
        // Slight debounce
        let mut last_event_time = Instant::now();

        while let Some(Ok(event)) = rx.recv().await {
            if last_event_time.elapsed().as_millis() < 100 {
                continue;
            }
            last_event_time = Instant::now();

            println!("CSS file changed, reloading styles.");
            if matches!(event.kind, EventKind::Modify(_)) {
                let _ = css_provider_clone.load_from_path("static/style.css");
            }
        }
        println!("CSS watcher stopped.");
    });

    gtk4::style_context_add_provider_for_display(
        &Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}