use evdev::Key;
use std::collections::HashMap;

pub static MOUSE_BTN_MAP: std::sync::LazyLock<HashMap<&'static str, Key>> =
    std::sync::LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("mouse_left_click", Key::BTN_LEFT);
        m.insert("mouse_right_click", Key::BTN_RIGHT);
        m.insert("mouse_middle_click", Key::BTN_MIDDLE);
        m.insert("mouse_back_click", Key::BTN_SIDE);
        m.insert("mouse_forward_click", Key::BTN_EXTRA);
        m.insert("advanced_click", Key::BTN_LEFT);
        m
    });

pub fn is_mouse_button_action(action_id: &str) -> bool {
    MOUSE_BTN_MAP.contains_key(action_id)
}

pub fn get_mouse_button_key(action_id: &str) -> Option<Key> {
    MOUSE_BTN_MAP.get(action_id).copied()
}
