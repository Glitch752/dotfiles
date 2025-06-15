use std::{collections::HashSet, hash::Hash};
use crate::{event::{InputEvent, KeyAction, MscType}, muxer::{MuxerServer, KEYBOARD_TAG, MOUSE_TAG}};

trait ContainsAll<T> {
    fn contains_all(&self, keys: &[T]) -> bool;
}
impl<T> ContainsAll<T> for HashSet<T> where T: Eq + Hash {
    fn contains_all(&self, keys: &[T]) -> bool {
        keys.iter().all(|key| self.contains(key))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Key {
    tag: u8,
    code: u16
}

// See https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
const KEY_LEFTALT: Key = Key { tag: KEYBOARD_TAG, code: 56 };
const KEY_LEFTMETA: Key = Key { tag: KEYBOARD_TAG, code: 125 };
const BTN_LEFT: Key = Key { tag: MOUSE_TAG, code: 0x110 };
const BTN_RIGHT: Key = Key { tag: MOUSE_TAG, code: 0x111 };

pub fn process() {
    let mut actual_pressed_keys: HashSet<Key> = HashSet::new();
    let mut simulated_pressed_keys: HashSet<Key> = HashSet::new();
    let mut desired_pressed_keys: HashSet<Key> = HashSet::new();

    let mut muxer = MuxerServer::new();

    while let Some(event) = muxer.read_input_event() {
        // EV_MSC / MSC_SCAN is used for remapping keys. We probably shouldn't receive this?
        if let InputEvent::Msc { ty: MscType::Scan, .. } = event {
            continue;
        }
        
        match event {
            InputEvent::Key { tag, key_code, action: KeyAction::Press, .. } => {
                actual_pressed_keys.insert(Key { tag, code: key_code });
            },
            InputEvent::Key { tag, key_code, action: KeyAction::Release, .. } => {
                actual_pressed_keys.remove(&Key { tag, code: key_code });
            },
            _ => {}
        }

        // Reset desired_pressed_keys to actual_pressed_keys
        desired_pressed_keys.clear();
        desired_pressed_keys.extend(actual_pressed_keys.iter());

        if actual_pressed_keys.contains_all(&[KEY_LEFTALT, KEY_LEFTMETA, BTN_LEFT]) {
            desired_pressed_keys.remove(&BTN_LEFT);
            desired_pressed_keys.insert(BTN_RIGHT);
            desired_pressed_keys.remove(&KEY_LEFTALT);
        }

        let mut passed_event = Some(event.clone());

        if let InputEvent::Key { tag, key_code, action: KeyAction::Release, .. } = &event {
            // Don't pass the event if it's a key release for a key that is still desired
            if desired_pressed_keys.contains(&Key { tag: *tag, code: *key_code }) {
                passed_event = None;
            }
        } else if let InputEvent::Key { tag, key_code, action: KeyAction::Press, .. } = &event {
            // Don't pass the event if it's a key press for a key that is not desired
            if !desired_pressed_keys.contains(&Key { tag: *tag, code: *key_code }) {
                passed_event = None;
            }
        }

        if let Some(event) = passed_event {
            match &event {
                InputEvent::Key { tag, key_code, action: KeyAction::Press, .. } => {
                    simulated_pressed_keys.insert(Key { tag: *tag, code: *key_code });
                },
                InputEvent::Key { tag, key_code, action: KeyAction::Release, .. } => {
                    simulated_pressed_keys.remove(&Key { tag: *tag, code: *key_code });
                },
                _ => {}
            }
            muxer.write_output_event(&event);
        }

        // Press all the keys that are desired but not currently simulated
        for key_code in desired_pressed_keys.difference(&simulated_pressed_keys.clone()) {
            muxer.write_output_event(&InputEvent::Key {
                tag: key_code.tag,
                key_code: key_code.code,
                action: KeyAction::Press,
                timestamp: event.timestamp()
            });
            simulated_pressed_keys.insert(*key_code);
        }
        // Release all the keys that are simulated but not currently desired
        for key_code in simulated_pressed_keys.clone().difference(&desired_pressed_keys) {
            muxer.write_output_event(&InputEvent::Key {
                tag: key_code.tag,
                key_code: key_code.code,
                action: KeyAction::Release,
                timestamp: event.timestamp()
            });
            simulated_pressed_keys.remove(key_code);
        }
    }
}