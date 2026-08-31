use crate::config::DeviceKey;
use super::Engine;
use crate::lock_ext::MutexExt;
use evdev::Key;

pub fn compute_blocked_buttons(
    mappings: &std::collections::HashMap<String, String>,
) -> (Vec<Key>, bool) {
    let mut blocked = Vec::new();
    let mut hscroll_blocked = false;
    for (btn_key, action) in mappings {
        let is_enabled_flag = btn_key.ends_with("_gesture_enabled");
        if action != "none" || (is_enabled_flag && action == "true") {
            if btn_key == "middle" || btn_key.starts_with("middle_gesture_") {
                if !blocked.contains(&Key::BTN_MIDDLE) {
                    blocked.push(Key::BTN_MIDDLE);
                }
            } else if btn_key == "xbutton1" || btn_key.starts_with("xbutton1_gesture_") {
                if !blocked.contains(&Key::BTN_SIDE) {
                    blocked.push(Key::BTN_SIDE);
                }
            } else if btn_key == "xbutton2" || btn_key.starts_with("xbutton2_gesture_") {
                if !blocked.contains(&Key::BTN_EXTRA) {
                    blocked.push(Key::BTN_EXTRA);
                }
            } else if btn_key == "hscroll_left" || btn_key == "hscroll_right" {
                hscroll_blocked = true;
            }
        }
    }
    (blocked, hscroll_blocked)
}

impl Engine {
    pub fn handle_app_change(&self, device: &DeviceKey, exe_name: String) {
        let (profile_name, mappings) = {
            let cfg = self.inner.config.lock_safe();
            let target = cfg.get_profile_for_app(device, &exe_name);
            let mappings = cfg.get_resolved_mappings(device, &target);
            (target, mappings)
        };

        let (changed, was_active) = {
            let mut map = self.inner.per_device.lock_safe();
            let st = map.entry(device.clone()).or_insert_with(Default::default);
            let changed_profile = st.current_profile != profile_name;
            let changed_exe = st.last_detected_exe != exe_name;
            if !changed_exe && !changed_profile {
                return;
            }
            st.last_detected_exe = exe_name.clone();
            if changed_profile {
                st.current_profile = profile_name.clone();
                st.active_profile_shared = profile_name.clone();
                let mappings_arc: std::collections::HashMap<String, std::sync::Arc<str>> = mappings
                    .iter()
                    .map(|(k, v)| (k.clone(), std::sync::Arc::from(v.as_str())))
                    .collect();
                *st.active_mappings.write().unwrap() = mappings_arc;
                let (blocked, hscroll_blocked) = compute_blocked_buttons(&mappings);
                st.blocked_buttons = blocked;
                st.block_hscroll = hscroll_blocked;
                (true, true)
            } else {
                (false, true)
            }
        };

        let _ = (changed, was_active);
        self.apply_keyboard_backlight(device);
    }
}
