pub mod actions;
pub mod compose;
pub mod emit;
pub mod hangul;
pub mod key_map;
pub mod layout_translator;
pub mod mouse_map;

pub use self::mouse_map::{get_mouse_button_key, is_mouse_button_action};

use crate::lock_ext::MutexExt;
use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, Key, RelativeAxisType,
};
use std::sync::{Arc, Mutex};

use self::mouse_map::MOUSE_BTN_MAP;

#[derive(Clone)]
pub struct KeySimulator {
    device: Arc<Mutex<Option<VirtualDevice>>>,
    keyboard_layout: Arc<Mutex<String>>,
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
            keyboard_layout: Arc::new(Mutex::new("ANSI (US)".to_string())),
        }
    }

    pub fn device(&self) -> Arc<Mutex<Option<VirtualDevice>>> {
        self.device.clone()
    }

    pub fn set_keyboard_layout(&self, layout: &str) {
        *self.keyboard_layout.lock_safe() = layout.to_string();
    }

    pub fn get_keyboard_layout(&self) -> String {
        self.keyboard_layout.lock_safe().clone()
    }

    pub fn ensure_device(&self) {
        let mut lock = self.device.lock_safe();
        if lock.is_none() {
            *lock = self.init_device();
        }
    }

    fn init_device(&self) -> Option<VirtualDevice> {
        let mut keys = AttributeSet::<Key>::new();
        // Insert all possible standard keys (1 to 248) and other codes up to 0x2ff
        for code in 1..=0x2ff {
            keys.insert(Key(code));
        }
        for &btn in (*MOUSE_BTN_MAP).values() {
            keys.insert(btn);
        }
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
        let mut lock = self.device.lock_safe();
        if lock.is_none() {
            *lock = self.init_device();
        }
        f(&mut lock)
    }
}
