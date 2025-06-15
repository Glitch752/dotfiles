use std::{collections::HashSet, hash::Hash, time::Duration};
use crate::{event::{AbsType, InputEvent, KeyAction, MscType, SynType}, muxer::{MuxerServer, KEYBOARD_TAG, MOUSE_TAG}};

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
const RIGHT_CLICK_POSITION: (i32, i32) = (1320, 860);

const EMULATE_TOUCHPAD: bool = true; // TODO: Make this configurable

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

        let override_with_right_click = actual_pressed_keys.contains_all(&[KEY_LEFTALT, KEY_LEFTMETA, BTN_LEFT]);
        if override_with_right_click {
            if !EMULATE_TOUCHPAD {
                desired_pressed_keys.remove(&BTN_LEFT);
            }
            desired_pressed_keys.insert(BTN_RIGHT);
            desired_pressed_keys.remove(&KEY_LEFTALT);
        }

        let mut passed_event = Some(event.clone());

        match &event {
            InputEvent::Key { tag, key_code, action: KeyAction::Release, .. } => {
                // Don't pass the event if it's a key release for a key that is still desired
                if desired_pressed_keys.contains(&Key { tag: *tag, code: *key_code }) {
                    passed_event = None;
                }
            }
            InputEvent::Key { tag, key_code, action: KeyAction::Press, .. } => {
                // Don't pass the event if it's a key press for a key that is not desired
                if !desired_pressed_keys.contains(&Key { tag: *tag, code: *key_code }) {
                    passed_event = None;
                }
            }
            _ => {}
        }

        if let Some(event) = &passed_event {
            match &event {
                InputEvent::Key { tag, key_code, action: KeyAction::Press, .. } => {
                    simulated_pressed_keys.insert(Key { tag: *tag, code: *key_code });
                },
                InputEvent::Key { tag, key_code, action: KeyAction::Release, .. } => {
                    simulated_pressed_keys.remove(&Key { tag: *tag, code: *key_code });
                },
                _ => {}
            }

            muxer.write_output_event(event);
        }

        // Press all the keys that are desired but not currently simulated
        for key_code in desired_pressed_keys.difference(&simulated_pressed_keys.clone()) {
            if key_code == &BTN_RIGHT && EMULATE_TOUCHPAD {
                // To begin a right-click drag on a touchpad, we start a multitouch event with one "finger" on the right-click position
                // See https://docs.kernel.org/input/multi-touch-protocol.html#protocol-example-b

                // First, use a (probably) unused ABS_MT slot to simulate a touchpad finger
                muxer.write_output_event(&InputEvent::Abs {
                    tag: key_code.tag,
                    ty: AbsType::MultitouchSlot,
                    value: 255,
                    timestamp: event.timestamp()
                });
                // Next, add a new "finger" with ABS_MT_TRACKING_ID. We use ID 0 because it's _probably_ currently unused.
                // ...I hope.
                muxer.write_output_event(&InputEvent::Abs {
                    tag: key_code.tag,
                    ty: AbsType::MultitouchTrackingId,
                    value: 0,
                    timestamp: event.timestamp()
                });
                // Finally, set the position of the "finger" to the right-click position
                muxer.write_output_event(&InputEvent::Abs {
                    tag: key_code.tag,
                    ty: AbsType::MultitouchPosX,
                    value: RIGHT_CLICK_POSITION.0,
                    timestamp: event.timestamp()
                });
                muxer.write_output_event(&InputEvent::Abs {
                    tag: key_code.tag,
                    ty: AbsType::MultitouchPosY,
                    value: RIGHT_CLICK_POSITION.1,
                    timestamp: event.timestamp()
                });
            } else {
                muxer.write_output_event(&InputEvent::Key {
                    tag: key_code.tag,
                    key_code: key_code.code,
                    action: KeyAction::Press,
                    timestamp: event.timestamp()
                });
            }

            simulated_pressed_keys.insert(*key_code);
        }
        // Release all the keys that are simulated but not currently desired
        for key_code in simulated_pressed_keys.clone().difference(&desired_pressed_keys) {
            if key_code == &BTN_RIGHT && EMULATE_TOUCHPAD {
                // To end a right click drag on a touchpad, we end the multitouch event by releasing the "finger"

                // Use our (probably) unused ABS_MT slot to simulate a touchpad finger
                muxer.write_output_event(&InputEvent::Abs {
                    tag: key_code.tag,
                    ty: AbsType::MultitouchSlot,
                    value: 255,
                    timestamp: event.timestamp()
                });
                // Release the "finger" by setting ABS_MT_TRACKING_ID to -1
                muxer.write_output_event(&InputEvent::Abs {
                    tag: key_code.tag,
                    ty: AbsType::MultitouchTrackingId,
                    value: -1,
                    timestamp: event.timestamp()
                });
            } else {
                muxer.write_output_event(&InputEvent::Key {
                    tag: key_code.tag,
                    key_code: key_code.code,
                    action: KeyAction::Release,
                    timestamp: event.timestamp()
                });
            }

            simulated_pressed_keys.remove(key_code);
        }
    }
}