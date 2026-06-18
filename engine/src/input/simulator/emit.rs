use evdev::{EventType, InputEvent, Key, RelativeAxisType};
use std::thread;
use std::time::Duration;

use super::KeySimulator;

impl KeySimulator {
    pub fn send_key_combo(&self, keys: &[Key], hold_ms: u64) {
        self.get_device(|dev_opt| {
            if let Some(dev) = dev_opt {
                let mut press_events: smallvec::SmallVec<[InputEvent; 8]> =
                    smallvec::SmallVec::new();
                for &key in keys {
                    press_events.push(InputEvent::new(EventType::KEY, key.0, 1));
                }
                press_events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));
                let _ = dev.emit(&press_events);

                if hold_ms > 0 {
                    thread::sleep(Duration::from_millis(hold_ms));
                }

                let mut release_events: smallvec::SmallVec<[InputEvent; 8]> =
                    smallvec::SmallVec::new();
                for &key in keys.iter().rev() {
                    release_events.push(InputEvent::new(EventType::KEY, key.0, 0));
                }
                release_events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));
                let _ = dev.emit(&release_events);
            }
        });
    }

    pub fn inject_mouse_click(&self, code: Key) {
        self.get_device(|dev_opt| {
            if let Some(dev) = dev_opt {
                let events = [
                    InputEvent::new(EventType::KEY, code.0, 1),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                    InputEvent::new(EventType::KEY, code.0, 0),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                ];
                let _ = dev.emit(&events);
            }
        });
    }

    pub fn inject_mouse_down(&self, code: Key) {
        self.get_device(|dev_opt| {
            if let Some(dev) = dev_opt {
                let events = [
                    InputEvent::new(EventType::KEY, code.0, 1),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                ];
                let _ = dev.emit(&events);
            }
        });
    }

    pub fn inject_mouse_up(&self, code: Key) {
        self.get_device(|dev_opt| {
            if let Some(dev) = dev_opt {
                let events = [
                    InputEvent::new(EventType::KEY, code.0, 0),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                ];
                let _ = dev.emit(&events);
            }
        });
    }

    pub fn inject_scroll(&self, horizontal: bool, delta: i32) {
        self.get_device(|dev_opt| {
            if let Some(dev) = dev_opt {
                let code = if horizontal {
                    RelativeAxisType::REL_HWHEEL
                } else {
                    RelativeAxisType::REL_WHEEL
                };

                let events = [
                    InputEvent::new(EventType::RELATIVE, code.0, delta),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                ];
                let _ = dev.emit(&events);
            }
        });
    }

    pub fn inject_relative_move(&self, dx: i32, dy: i32) {
        self.get_device(|dev_opt| {
            if let Some(dev) = dev_opt {
                let events = [
                    InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_X.0, dx),
                    InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_Y.0, dy),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                ];
                let _ = dev.emit(&events);
            }
        });
    }

    pub fn inject_key_up(&self, key: evdev::Key) {
        self.get_device(|dev_opt| {
            if let Some(dev) = dev_opt {
                let events = [
                    InputEvent::new(EventType::KEY, key.0, 0),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                ];
                let _ = dev.emit(&events);
            }
        });
    }

    #[allow(clippy::vec_init_then_push)]
    pub fn type_unicode_char(&self, ch: char) {
        let hex = format!("{:x}", ch as u32);
        self.get_device(|dev_opt| {
            if let Some(dev) = dev_opt {
                let mut events = Vec::new();

                // 1. Press Ctrl + Shift + U
                events.push(InputEvent::new(EventType::KEY, Key::KEY_LEFTCTRL.0, 1));
                events.push(InputEvent::new(EventType::KEY, Key::KEY_LEFTSHIFT.0, 1));
                events.push(InputEvent::new(EventType::KEY, Key::KEY_U.0, 1));
                events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));

                // Release U, Shift, Ctrl
                events.push(InputEvent::new(EventType::KEY, Key::KEY_U.0, 0));
                events.push(InputEvent::new(EventType::KEY, Key::KEY_LEFTSHIFT.0, 0));
                events.push(InputEvent::new(EventType::KEY, Key::KEY_LEFTCTRL.0, 0));
                events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));

                // 2. Type hex digits
                for c in hex.chars() {
                    if let Some(key) = hex_char_to_key(c) {
                        events.push(InputEvent::new(EventType::KEY, key.0, 1));
                        events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));
                        events.push(InputEvent::new(EventType::KEY, key.0, 0));
                        events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));
                    }
                }

                // 3. Press Enter to commit
                events.push(InputEvent::new(EventType::KEY, Key::KEY_ENTER.0, 1));
                events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));
                events.push(InputEvent::new(EventType::KEY, Key::KEY_ENTER.0, 0));
                events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));

                let _ = dev.emit(&events);
            }
        });
    }
}

fn hex_char_to_key(c: char) -> Option<Key> {
    match c {
        '0' => Some(Key::KEY_0),
        '1' => Some(Key::KEY_1),
        '2' => Some(Key::KEY_2),
        '3' => Some(Key::KEY_3),
        '4' => Some(Key::KEY_4),
        '5' => Some(Key::KEY_5),
        '6' => Some(Key::KEY_6),
        '7' => Some(Key::KEY_7),
        '8' => Some(Key::KEY_8),
        '9' => Some(Key::KEY_9),
        'a' | 'A' => Some(Key::KEY_A),
        'b' | 'B' => Some(Key::KEY_B),
        'c' | 'C' => Some(Key::KEY_C),
        'd' | 'D' => Some(Key::KEY_D),
        'e' | 'E' => Some(Key::KEY_E),
        'f' | 'F' => Some(Key::KEY_F),
        _ => None,
    }
}
