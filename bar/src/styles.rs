// TODO: Hot reload CSS

use gtk4::gdk::Display;

pub fn load_css() {
    let css_provider = gtk4::CssProvider::new();
    let css_file = std::path::PathBuf::from(env!("OUT_DIR")).join("style.css");
    css_provider.load_from_path(css_file);


    gtk4::style_context_add_provider_for_display(
        &Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}