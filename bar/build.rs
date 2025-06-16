use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let scss_input = "assets/style.scss";
    let css_output = Path::new(&out_dir).join("style.css");

    // Instruct Cargo to rerun this script if any SCSS file changes
    println!("cargo:rerun-if-changed=assets/*.scss");

    // Run the 'sass' command to compile SCSS to CSS
    let status = Command::new("sass")
        .arg("--no-error-css") // optional: don't emit CSS if there's an error
        .arg(scss_input)
        .arg(css_output.to_str().unwrap())
        .status()
        .expect("Failed to execute 'sass' command");

    if !status.success() {
        panic!("SCSS compilation failed");
    }
}
