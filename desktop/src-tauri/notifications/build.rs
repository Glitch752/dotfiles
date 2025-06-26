const COMMANDS: &[&str] = &[];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();

    // Never rerun unless build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}
