pub mod key_map;
pub mod mouse_map;
pub mod emit;
pub mod actions;

pub use self::mouse_map::{is_mouse_button_action, get_mouse_button_key};

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, Key, RelativeAxisType,
};
use std::sync::{Arc, Mutex};

use self::key_map::KEY_MAP;
use self::mouse_map::MOUSE_BTN_MAP;

#[derive(Clone)]
pub struct KeySimulator {
    device: Arc<Mutex<Option<VirtualDevice>>>,
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
        let mut keys = AttributeSet::<Key>::new();
        for &key in (*KEY_MAP).values() {
            keys.insert(key);
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
        let mut lock = self.device.lock().unwrap();
        if lock.is_none() {
            *lock = self.init_device();
        }
        f(&mut lock)
    }
}
