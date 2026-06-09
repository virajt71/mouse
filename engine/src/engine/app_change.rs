use evdev::Key;
use std::sync::atomic::Ordering;
use super::Engine;

pub fn compute_blocked_buttons(mappings: &std::collections::HashMap<String, String>) -> (Vec<Key>, bool) {
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
    pub fn handle_app_change(&self, exe_name: String) {
        let (profile_name, mappings) = {
            let cfg = self.inner.config.lock().unwrap();
            let target = cfg.get_profile_for_app(&exe_name);
            let mappings = cfg.get_resolved_mappings(&target);
            (target, mappings)
        };

        let mut last_exe = self.inner.last_detected_exe.lock().unwrap();
        let mut current_profile = self.inner.current_profile.lock().unwrap();
        
        if *current_profile == profile_name && *last_exe == exe_name {
            return;
        }
        
        *last_exe = exe_name.clone();
        
        if *current_profile != profile_name {
            log::info!("[Engine] App {} → profile '{}'", exe_name, profile_name);
            *current_profile = profile_name.clone();
            drop(current_profile);
            drop(last_exe);
            
            {
                let mut cfg = self.inner.config.lock().unwrap();
                cfg.active_profile = profile_name.clone();
                let _ = cfg.save();
                self.increment_config_generation();
            }
            
            *self.inner.active_profile_shared.lock().unwrap() = profile_name;
            *self.inner.active_mappings.lock().unwrap() = mappings.clone();
            let (blocked, hscroll_blocked) = compute_blocked_buttons(&mappings);
            *self.inner.blocked_buttons_arc.lock().unwrap() = blocked;
            self.inner.block_hscroll_arc.store(hscroll_blocked, Ordering::SeqCst);
            let _ = self.restart_keyboard_hooks();
        }
    }
}
