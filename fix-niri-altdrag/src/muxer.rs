use crate::{event::{read_event, write_event, InputEvent}, muxer::fifo::FifoQueue};

pub mod fifo;

pub const KEYBOARD_TAG: u8 = 0;
pub const MOUSE_TAG: u8 = 1;

const INPUT_FIFO_PATH: &str = "/tmp/fix-niri-altdrag-input";
const OUTPUT_FIFO_PATH: &str = "/tmp/fix-niri-altdrag-output-"; // This is a prefix, the tag will be appended to it.
const REGISTRATION_FIFO_PATH: &str = "/tmp/fix-niri-altdrag-registration";

static TAG_NAME_MAP: &[(&str, u8)] = &[
    ("kbd", KEYBOARD_TAG),
    ("mouse", MOUSE_TAG),
];

fn get_tag_from_name(name: &str) -> Option<u8> {
    TAG_NAME_MAP.iter().find_map(|&(n, tag)| if n == name { Some(tag) } else { None })
}

/// MuxerServer manages the pipeline from input -> processing -> output.
/// We have two types of IPC channels:
/// - Input channel: Reads input events from stdin on each "in" instance and writes them to the IPC queue.
/// - Output channel: The processing logic reads events from the IPC queue and moves them to the output IPC queue.
///   From there, each "out" instance reads events from the IPC queue and writes them to stdout.
/// The server creates a FIFO queue for the instance and stores it in a map.
pub struct MuxerServer {
    // Multi-producer, single-consumer FIFO queue for input events:
    // we only need one FIFO since Unix (normally) serializes small writes
    input_queue: FifoQueue<InputEvent>,
    // Multi-consumer, single-producer FIFO queues for output events:
    // we need one FIFO for each output instance.
    output_queues: std::collections::HashMap<u8, FifoQueue<InputEvent>>,
    // The registration FIFO queue is used to register new output instances.
    registration_queue: FifoQueue<u8>
}

impl MuxerServer {
    pub fn new() -> Self {
        MuxerServer {
            input_queue: FifoQueue::new(INPUT_FIFO_PATH).expect("Failed to create input FIFO"),
            output_queues: std::collections::HashMap::new(),
            registration_queue: FifoQueue::new(REGISTRATION_FIFO_PATH).expect("Failed to create registration FIFO"),
        }
    }

    pub fn read_input_event(&mut self) -> Option<InputEvent> {
        // This is a bit sketchy, but since this is called in a loop, we can update registration queues too.
        loop {
            if self.registration_queue.has_available() {
                match self.registration_queue.read(-1) {
                    Ok(tag) => {
                        if self.output_queues.get_mut(&tag).is_some() {
                            // If the queue already exists, we can just continue.
                            // This is unsupported, but probably not fatal (?)
                            eprintln!("ERROR: Output queue for tag {} already exists", tag);
                        } else {
                            // Create a new output queue for the tag
                            let new_queue = FifoQueue::new(&format!("{}{}", OUTPUT_FIFO_PATH, tag))
                                .expect("Failed to create output FIFO");
                            self.output_queues.insert(tag, new_queue);
                            eprintln!("Created new output queue for tag {}", tag);
                        }
                    },
                    Err(_) => eprintln!("Failed to read from registration FIFO"),
                }
            }

            match self.input_queue.read(10) {
                Ok(event) => return Some(event),
                Err(_) => {
                    // This is expected--at least a few times at startup.
                }
            }
        }
    }

    pub fn write_output_event(&mut self, event: &InputEvent) {
        // fn code_to_string(code: u16) -> String {
        //     match code {
        //         0x110 => "BTN_LEFT".to_string(),
        //         0x111 => "BTN_RIGHT".to_string(),
        //         330 => "BTN_TOUCH".to_string(),
        //         325 => "BTN_TOOL_FINGER".to_string(),
        //         _ => format!("{}", code),
        //     }
        // }
        // if let InputEvent::Key { tag, key_code, action, .. } = event {
        //     if *tag == MOUSE_TAG {
        //         eprintln!("Synthetic key release: {}", code_to_string(*key_code));
        //     }
        // }

        if let Some(queue) = self.output_queues.get_mut(&event.tag()) {
            queue.write(event).expect("Failed to write to output FIFO");
        } else {
            eprintln!("No output queue for tag {}", event.tag());
        }
    }
}

pub fn input(tag: String) {
    let tag = match get_tag_from_name(&tag) {
        Some(tag) => tag,
        None => {
            eprintln!("Unknown tag: {}", tag);
            std::process::exit(1);
        }
    };

    let mut stdin = std::io::stdin().lock();

    let mut input_fifo = FifoQueue::new(INPUT_FIFO_PATH).expect("Failed to open input FIFO");
    while let Some(event) = read_event(&mut stdin, tag) {
        input_fifo.write(&event).expect("Failed to write to input FIFO");
    }
}

pub fn output(tag: String) {
    let tag = match get_tag_from_name(&tag) {
        Some(tag) => tag,
        None => {
            eprintln!("Unknown tag: {}", tag);
            std::process::exit(1);
        }
    };

    let mut stdout = std::io::stdout().lock();

    let mut output_fifo = FifoQueue::new(&format!("{}{}", OUTPUT_FIFO_PATH, tag))
        .expect("Failed to open output FIFO");

    let mut registration_fifo = FifoQueue::new(REGISTRATION_FIFO_PATH)
        .expect("Failed to open registration FIFO");

    // Register this instance in the registration FIFO
    registration_fifo.write(&tag).expect("Failed to write to registration FIFO");

    eprintln!("Registered output instance with tag {}", tag);

    while let Ok(event) = output_fifo.read(-1) {
        write_event(&mut stdout, &event);
    }
}