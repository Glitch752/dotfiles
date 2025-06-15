use std::env;

mod process;
mod muxer;
mod event;

// TODO: Restructure this to have fewer edge cases and more clear logic.

// Note: Do not use println!() in this program, as it will be piped to stdout. Use eprintln!() for debugging output instead.

fn main() {
    // If the program is called with the "in" subcommand, it relays events from stdin to the ipc queue with the specified tag.
    // If the program is called with the "out" subcommand, it relays events from the ipc queue with the specified tag to stdout.
    // Otherwise, runs the normal processing logic.
    if env::args().len() == 1 {
        // Normal processing logic
        process::process();
    } else if env::args().nth(1) == Some("in".to_string()) {
        // Tagging logic
        if let Some(tag) = env::args().nth(2) {
            muxer::input(tag);
        } else {
            eprintln!("No tag value provided.");
            std::process::exit(1);
        }
    } else if env::args().nth(1) == Some("out".to_string()) {
        // Filtering logic
        if let Some(tag) = env::args().nth(2) {
            muxer::output(tag);
        } else {
            eprintln!("No tag value provided.");
            std::process::exit(1);
        }
    } else {
        eprintln!("Unknown command. Use 'in <value>' to input events or 'out <value>' to output events.");
        std::process::exit(1);
    }
}
