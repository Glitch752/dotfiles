use std::process::Command;

fn main() {
    let scss_input = "assets/style.scss";
    let css_output = "static/style.css";

    // Only rerun if an SCSS file under assets/ changes
    println!("cargo:rerun-if-changed=assets/**/*.scss");

    // Run the 'sass' command to compile SCSS to CSS
    let status = Command::new("sass")
        .arg("--no-error-css") // optional: don't emit CSS if there's an error
        .arg(scss_input)
        .arg(css_output)
        .status()
        .expect("Failed to execute 'sass' command");

    if !status.success() {
        panic!("SCSS compilation failed");
    }
}
