use std::{io::{Read, Write}, time::{Duration, SystemTime}};
use std::mem::size_of;

use crate::muxer::fifo::TrustMeBroThisIsSafe;

/// See https://docs.kernel.org/input/event-codes.html
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RawInputEvent {
    pub time: libc::timeval,
    pub type_: u16,
    pub code: u16,
    pub value: i32
}

impl RawInputEvent {
    /// Gets the timestmap of the event as a `SystemTime`.
    pub fn timestamp(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH
            .checked_add(Duration::new(
                self.time.tv_sec as u64,
                (self.time.tv_usec as u32) * 1000, // 1us = 1000ns
            ))
            .unwrap_or(SystemTime::UNIX_EPOCH)
    }
}

// See https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
const EV_SYN: u16 = 0x00; // Separator between multiple events
const EV_KEY: u16 = 0x01; // Changes of keyboards, buttons, etc
// const EV_REL: u16 = 0x02; // Relative axis movement
// const EV_ABS: u16 = 0x03; // Absolute axis movement
const EV_MSC: u16 = 0x04; // Miscellaneous

// A slightly-safer wrapper over RawInputEvent.
#[derive(Clone)]
pub enum InputEvent {
    /// EV_SYN event values are undefined. Their usage is defined only by when they are sent in the evdev event stream.
    Syn {
        tag: u8,
        timestamp: SystemTime,
        code: u16
    },
    Key {
        tag: u8,
        timestamp: SystemTime,
        key_code: u16,
        action: KeyAction
    },
    Msc {
        tag: u8,
        timestamp: SystemTime,
        ty: MscType,
        value: i32
    },
    Unknown {
        tag: u8,
        event: RawInputEvent
    }
}

unsafe impl TrustMeBroThisIsSafe for InputEvent {}

impl InputEvent {
    /// Returns the timestamp of the event.
    pub fn timestamp(&self) -> SystemTime {
        match self {
            InputEvent::Syn { timestamp, .. } => *timestamp,
            InputEvent::Key { timestamp, .. } => *timestamp,
            InputEvent::Msc { timestamp, .. } => *timestamp,
            InputEvent::Unknown { event, .. } => event.timestamp()
        }
    }

    pub fn tag(&self) -> u8 {
        match self {
            InputEvent::Syn { tag, .. } => *tag,
            InputEvent::Key { tag, .. } => *tag,
            InputEvent::Msc { tag, .. } => *tag,
            InputEvent::Unknown { tag, .. } => *tag,
        }
    }
}

// Code is the type of miscellaneous event
#[derive(Clone, PartialEq)]
pub enum MscType {
    Scan, // MSC_SCAN
    Other(u16)
}

impl From<u16> for MscType {
    fn from(value: u16) -> Self {
        match value {
            0x04 => MscType::Scan,
            other => MscType::Other(other)
        }
    }
}

impl Into<u16> for MscType {
    fn into(self) -> u16 {
        match self {
            MscType::Scan => 0x04,
            MscType::Other(code) => code
        }
    }
}

const KEY_RELEASE: i32 = 0;
const KEY_PRESS: i32 = 1;
const KEY_AUTOREPEAT: i32 = 2;

// Value is 0 for EV_KEY for release, 1 for keypress and 2 for autorepeat.
#[derive(Clone, PartialEq)]
pub enum KeyAction {
    Release,
    Press,
    Autorepeat,
    Unknown(i32)
}

impl From<i32> for KeyAction {
    fn from(value: i32) -> Self {
        match value {
            KEY_RELEASE => KeyAction::Release,
            KEY_PRESS => KeyAction::Press,
            KEY_AUTOREPEAT => KeyAction::Autorepeat,
            other => KeyAction::Unknown(other),
        }
    }
}

impl Into<i32> for KeyAction {
    fn into(self) -> i32 {
        match self {
            KeyAction::Release => KEY_RELEASE,
            KeyAction::Press => KEY_PRESS,
            KeyAction::Autorepeat => KEY_AUTOREPEAT,
            KeyAction::Unknown(val) => val,
        }
    }
}

fn system_time_to_timeval(timestamp: SystemTime) -> libc::timeval {
    let duration = timestamp
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    libc::timeval {
        tv_sec: duration.as_secs() as libc::time_t,
        tv_usec: (duration.subsec_nanos() / 1000) as libc::suseconds_t, // 1000ns = 1us
    }
}

impl Into<RawInputEvent> for InputEvent {
    fn into(self) -> RawInputEvent {
        match self {
            InputEvent::Syn { code, timestamp, .. } => {
                RawInputEvent {
                    type_: EV_SYN,
                    code,
                    time: system_time_to_timeval(timestamp),
                    value: 0 // Value is undefined
                }
            }
            InputEvent::Key { key_code, action, timestamp, .. } => {
                RawInputEvent {
                    type_: EV_KEY,
                    code: key_code,
                    time: system_time_to_timeval(timestamp),
                    value: action.into()
                }
            }
            InputEvent::Msc { ty, value, timestamp, .. } => {
                RawInputEvent {
                    type_: EV_MSC,
                    code: ty.into(),
                    time: system_time_to_timeval(timestamp),
                    value
                }
            }
            InputEvent::Unknown { event, .. } => event
        }
    }
}

impl InputEvent {
    fn from_raw(event: RawInputEvent, tag: u8) -> Self {
        let timestamp = event.timestamp();
        match(event.type_, event.code) {
            (EV_SYN, code) => InputEvent::Syn { tag, timestamp, code },
            (EV_KEY, key_code) => InputEvent::Key { tag, timestamp, key_code, action: event.value.into() },
            (EV_MSC, code) => InputEvent::Msc { tag, timestamp, ty: code.into(), value: event.value.into() },
            _ => InputEvent::Unknown { tag, event }
        }
    }
}

pub fn read_raw_event(stdin: &mut impl Read) -> Option<RawInputEvent> {
    let mut buffer = [0u8; size_of::<RawInputEvent>()];
    if stdin.read_exact(&mut buffer).is_ok() {
        let ptr = buffer.as_ptr() as *const RawInputEvent;
        // Safety: This is probably fine, idk (best safety guarentees)
        unsafe { Some(*ptr) }
    } else {
        None
    }
}

pub fn write_raw_event(stdout: &mut impl Write, event: &RawInputEvent) {
    let bytes = unsafe {
        // Safety: Length is correct
        std::slice::from_raw_parts(
            event as *const RawInputEvent as *const u8,
            size_of::<RawInputEvent>(),
        )
    };
    if stdout.write_all(bytes).is_err() {
        std::process::exit(1);
    }
    stdout.flush().expect("Failed to flush stdout");
}

pub fn read_event(reader: &mut impl Read, tag_as: u8) -> Option<InputEvent> {
    read_raw_event(reader).map(|raw_event| {
        InputEvent::from_raw(raw_event, tag_as)
    })
}

pub fn write_event(writer: &mut impl Write, event: &InputEvent) {
    let raw_event: RawInputEvent = event.clone().into();
    write_raw_event(writer, &raw_event);
}