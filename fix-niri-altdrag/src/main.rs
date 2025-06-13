use std::collections::HashSet;
use std::thread;
use std::time::Duration;

const LWIN: u16 = 0xE0;
const LALT: u16 = 0x38;
const LMB: u16 = 0x110;
const RMB: u16 = 0x111;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Track the state of the modifiers and mouse buttons
    let mut modifiers = HashSet::new();
    let mut mouse_buttons = HashSet::new();

    loop {
        // TODO

        // Sleep to prevent high CPU usage
        thread::sleep(Duration::from_millis(5));
    }
}