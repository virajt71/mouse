use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, EventType, InputEvent, Key, RelativeAxisType,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct KeySimulator {
    device: Arc<Mutex<Option<VirtualDevice>>>,
}

lazy_static::lazy_static! {
    static ref KEY_MAP: HashMap<&'static str, Key> = {
        let mut m = HashMap::new();
        m.insert("ctrl", Key::KEY_LEFTCTRL);
        m.insert("crtl", Key::KEY_LEFTCTRL);
        m.insert("control", Key::KEY_LEFTCTRL);
        m.insert("shift", Key::KEY_LEFTSHIFT);
        m.insert("alt", Key::KEY_LEFTALT);
        m.insert("opt", Key::KEY_LEFTALT);
        m.insert("option", Key::KEY_LEFTALT);
        m.insert("super", Key::KEY_LEFTMETA);
        m.insert("meta", Key::KEY_LEFTMETA);
        m.insert("cmd", Key::KEY_LEFTMETA);
        m.insert("command", Key::KEY_LEFTMETA);
        m.insert("win", Key::KEY_LEFTMETA);
        m.insert("windows", Key::KEY_LEFTMETA);
        m.insert("tab", Key::KEY_TAB);
        m.insert("space", Key::KEY_SPACE);
        m.insert("enter", Key::KEY_ENTER);
        m.insert("esc", Key::KEY_ESC);
        m.insert("backspace", Key::KEY_BACKSPACE);
        m.insert("delete", Key::KEY_DELETE);
        m.insert("left", Key::KEY_LEFT);
        m.insert("right", Key::KEY_RIGHT);
        m.insert("up", Key::KEY_UP);
        m.insert("down", Key::KEY_DOWN);
        m.insert("pageup", Key::KEY_PAGEUP);
        m.insert("pagedown", Key::KEY_PAGEDOWN);
        m.insert("home", Key::KEY_HOME);
        m.insert("end", Key::KEY_END);
        m.insert("volumeup", Key::KEY_VOLUMEUP);
        m.insert("volumedown", Key::KEY_VOLUMEDOWN);
        m.insert("mute", Key::KEY_MUTE);
        m.insert("playpause", Key::KEY_PLAYPAUSE);
        m.insert("nexttrack", Key::KEY_NEXTSONG);
        m.insert("prevtrack", Key::KEY_PREVIOUSSONG);

        // A-Z
        m.insert("a", Key::KEY_A);
        m.insert("b", Key::KEY_B);
        m.insert("c", Key::KEY_C);
        m.insert("d", Key::KEY_D);
        m.insert("e", Key::KEY_E);
        m.insert("f", Key::KEY_F);
        m.insert("g", Key::KEY_G);
        m.insert("h", Key::KEY_H);
        m.insert("i", Key::KEY_I);
        m.insert("j", Key::KEY_J);
        m.insert("k", Key::KEY_K);
        m.insert("l", Key::KEY_L);
        m.insert("m", Key::KEY_M);
        m.insert("n", Key::KEY_N);
        m.insert("o", Key::KEY_O);
        m.insert("p", Key::KEY_P);
        m.insert("q", Key::KEY_Q);
        m.insert("r", Key::KEY_R);
        m.insert("s", Key::KEY_S);
        m.insert("t", Key::KEY_T);
        m.insert("u", Key::KEY_U);
        m.insert("v", Key::KEY_V);
        m.insert("w", Key::KEY_W);
        m.insert("x", Key::KEY_X);
        m.insert("y", Key::KEY_Y);
        m.insert("z", Key::KEY_Z);

        // 0-9
        m.insert("0", Key::KEY_0);
        m.insert("1", Key::KEY_1);
        m.insert("2", Key::KEY_2);
        m.insert("3", Key::KEY_3);
        m.insert("4", Key::KEY_4);
        m.insert("5", Key::KEY_5);
        m.insert("6", Key::KEY_6);
        m.insert("7", Key::KEY_7);
        m.insert("8", Key::KEY_8);
        m.insert("9", Key::KEY_9);

        // F1-F12
        m.insert("f1", Key::KEY_F1);
        m.insert("f2", Key::KEY_F2);
        m.insert("f3", Key::KEY_F3);
        m.insert("f4", Key::KEY_F4);
        m.insert("f5", Key::KEY_F5);
        m.insert("f6", Key::KEY_F6);
        m.insert("f7", Key::KEY_F7);
        m.insert("f8", Key::KEY_F8);
        m.insert("f9", Key::KEY_F9);
        m.insert("f10", Key::KEY_F10);
        m.insert("f11", Key::KEY_F11);
        m.insert("f12", Key::KEY_F12);
        m.insert("equal", Key::KEY_EQUAL);
        m.insert("minus", Key::KEY_MINUS);
        m.insert("brightnessup", Key::KEY_BRIGHTNESSUP);
        m.insert("brightnessdown", Key::KEY_BRIGHTNESSDOWN);

        m
    };

    static ref MOUSE_BTN_MAP: HashMap<&'static str, Key> = {
        let mut m = HashMap::new();
        m.insert("mouse_left_click", Key::BTN_LEFT);
        m.insert("mouse_right_click", Key::BTN_RIGHT);
        m.insert("mouse_middle_click", Key::BTN_MIDDLE);
        m.insert("mouse_back_click", Key::BTN_SIDE);
        m.insert("mouse_forward_click", Key::BTN_EXTRA);
        m.insert("advanced_click", Key::BTN_LEFT);
        m
    };
}

pub fn is_mouse_button_action(action_id: &str) -> bool {
    MOUSE_BTN_MAP.contains_key(action_id)
}

pub fn get_mouse_button_key(action_id: &str) -> Option<Key> {
    MOUSE_BTN_MAP.get(action_id).copied()
}


impl Default for KeySimulator {
    fn default() -> Self {
        Self::new()
    }
}

impl KeySimulator {
    pub fn new() -> Self {
        KeySimulator {
            device: Arc::new(Mutex::new(None)),
        }
    }

    pub fn device(&self) -> Arc<Mutex<Option<VirtualDevice>>> {
        self.device.clone()
    }

    pub fn ensure_device(&self) {
        let mut lock = self.device.lock().unwrap();
        if lock.is_none() {
            *lock = self.init_device();
        }
    }

    fn init_device(&self) -> Option<VirtualDevice> {
        // Collect all keys to register
        let mut keys = AttributeSet::<Key>::new();
        for &key in (*KEY_MAP).values() {
            keys.insert(key);
        }
        for &btn in (*MOUSE_BTN_MAP).values() {
            keys.insert(btn);
        }
        // Also register standard browser navigation keys if not in KEY_MAP
        keys.insert(Key::KEY_BACK);
        keys.insert(Key::KEY_FORWARD);

        let mut relative_axes = AttributeSet::<RelativeAxisType>::new();
        relative_axes.insert(RelativeAxisType::REL_X);
        relative_axes.insert(RelativeAxisType::REL_Y);
        relative_axes.insert(RelativeAxisType::REL_WHEEL);
        relative_axes.insert(RelativeAxisType::REL_HWHEEL);

        match VirtualDeviceBuilder::new()
            .ok()?
            .name("Mouser Virtual Keyboard")
            .with_keys(&keys)
            .ok()?
            .with_relative_axes(&relative_axes)
            .ok()?
            .build()
        {
            Ok(dev) => {
                log::info!("[KeySimulator] Virtual device created successfully");
                Some(dev)
            }
            Err(e) => {
                log::error!(
                    "[KeySimulator] Failed to create virtual keyboard: {}. Ensure /dev/uinput is writable.",
                    e
                );
                None
            }
        }
    }

    fn get_device<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Option<VirtualDevice>) -> R,
    {
        let mut lock = self.device.lock().unwrap();
        if lock.is_none() {
            *lock = self.init_device();
        }
        f(&mut lock)
    }

    pub fn send_key_combo(&self, keys: &[Key], hold_ms: u64) {
        self.get_device(|dev_opt| {
            if let Some(dev) = dev_opt {
                // Press keys in order
                let mut press_events = Vec::new();
                for &key in keys {
                    press_events.push(InputEvent::new(EventType::KEY, key.0, 1));
                }
                press_events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));
                let _ = dev.emit(&press_events);

                if hold_ms > 0 {
                    thread::sleep(Duration::from_millis(hold_ms));
                }

                // Release keys in reverse order
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

    pub fn execute_action(&self, action_id: &str) {
        log::debug!("[KeySimulator] Executing action: {}", action_id);
        if action_id == "none" {
            return;
        }


        if let Some(combo) = action_id.strip_prefix("custom:") {
            let mut keys = Vec::new();
            for part in combo.split('+') {
                let clean = part.trim().to_lowercase();
                if let Some(&key) = (*KEY_MAP).get(clean.as_str()) {
                    keys.push(key);
                } else {
                    log::warn!("[KeySimulator] Unknown custom key name: {}", clean);
                }
            }
            if !keys.is_empty() {
                self.send_key_combo(&keys, 50);
            }
            return;
        }

        if let Some(&btn) = (*MOUSE_BTN_MAP).get(action_id) {
            self.inject_mouse_click(btn);
            return;
        }

        // Standard action mappings
        let keys_opt = match action_id {
            "alt_tab" => Some(vec![Key::KEY_LEFTALT, Key::KEY_TAB]),
            "alt_shift_tab" => Some(vec![Key::KEY_LEFTALT, Key::KEY_LEFTSHIFT, Key::KEY_TAB]),
            "browser_back" => Some(vec![Key::KEY_BACK]),
            "browser_forward" => Some(vec![Key::KEY_FORWARD]),
            "copy" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_C]),
            "paste" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_V]),
            "cut" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_X]),
            "undo" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_Z]),
            "select_all" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_A]),
            "save" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_S]),
            "close_tab" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_W]),
            "new_tab" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_T]),
            "find" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_F]),
            "win_d" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_D]),
            "task_view" => Some(vec![Key::KEY_LEFTMETA]),
            "volume_up" => Some(vec![Key::KEY_VOLUMEUP]),
            "volume_down" => Some(vec![Key::KEY_VOLUMEDOWN]),
            "volume_mute" => Some(vec![Key::KEY_MUTE]),
            "play_pause" => Some(vec![Key::KEY_PLAYPAUSE]),
            "next_track" => Some(vec![Key::KEY_NEXTSONG]),
            "prev_track" => Some(vec![Key::KEY_PREVIOUSSONG]),
            "page_up" => Some(vec![Key::KEY_PAGEUP]),
            "page_down" => Some(vec![Key::KEY_PAGEDOWN]),
            "home" => Some(vec![Key::KEY_HOME]),
            "end" => Some(vec![Key::KEY_END]),
            // Desktops/Workspaces switching
            "space_left" => {
                // KDE/Plasma standard fallback
                Some(vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTALT, Key::KEY_LEFT])
            }
            "space_right" => {
                Some(vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTALT, Key::KEY_RIGHT])
            }
            "zoom_in" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_EQUAL]),
            "zoom_out" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_MINUS]),
            "tab_prev" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_PAGEUP]),
            "tab_next" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_PAGEDOWN]),
            "brightness_up" => Some(vec![Key::KEY_BRIGHTNESSUP]),
            "brightness_down" => Some(vec![Key::KEY_BRIGHTNESSDOWN]),
            "app_prev" => Some(vec![Key::KEY_LEFTALT, Key::KEY_LEFTSHIFT, Key::KEY_TAB]),
            "app_next" => Some(vec![Key::KEY_LEFTALT, Key::KEY_TAB]),
            "screen_capture" => Some(vec![Key::KEY_SYSRQ]),
            "print_screen" => Some(vec![Key::KEY_SYSRQ]),
            "action_center" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_A]),
            "calculator" => Some(vec![Key::KEY_CALC]),
            "close_window" => Some(vec![Key::KEY_LEFTALT, Key::KEY_F4]),
            "dictation" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_H]),
            "emoji" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_DOT]),
            "emojis_menu" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_DOT]),
            "input_language" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_SPACE]),
            "lock" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_L]),
            "maximize_window" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_UP]),
            "minimize_window" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_DOWN]),
            "open_application" => Some(vec![Key::KEY_LEFTMETA]),
            "open_file" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_O]),
            "open_folder" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_E]),
            "redo" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_Y]),
            "right_ctrl" => Some(vec![Key::KEY_RIGHTCTRL]),
            "screen_snip" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_LEFTSHIFT, Key::KEY_S]),
            "this_pc" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_E]),
            "snap_left" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_LEFT]),
            "snap_right" => Some(vec![Key::KEY_LEFTMETA, Key::KEY_RIGHT]),
            "pan_left" => Some(vec![Key::KEY_LEFT]),
            "pan_right" => Some(vec![Key::KEY_RIGHT]),
            "pan_up" => Some(vec![Key::KEY_UP]),
            "pan_down" => Some(vec![Key::KEY_DOWN]),
            "rotate_left" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT, Key::KEY_R]),
            "rotate_right" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_R]),
            "zoom_reset" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_0]),
            "start_menu" => Some(vec![Key::KEY_LEFTMETA]),
            _ => None,
        };

        if let Some(keys) = keys_opt {
            self.send_key_combo(&keys, 50);
        }
    }
}
