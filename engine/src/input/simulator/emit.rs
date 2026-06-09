use evdev::{EventType, InputEvent, Key, RelativeAxisType};
use std::thread;
use std::time::Duration;

use super::KeySimulator;

impl KeySimulator {
    pub fn send_key_combo(&self, keys: &[Key], hold_ms: u64) {
        self.get_device(|dev_opt| {
            if let Some(dev) = dev_opt {
                let mut press_events = Vec::new();
                for &key in keys {
                    press_events.push(InputEvent::new(EventType::KEY, key.0, 1));
                }
                press_events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));
                let _ = dev.emit(&press_events);

                if hold_ms > 0 {
                    thread::sleep(Duration::from_millis(hold_ms));
                }

                let mut release_events = Vec::new();
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
}
