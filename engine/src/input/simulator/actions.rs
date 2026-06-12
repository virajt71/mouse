use super::mouse_map::MOUSE_BTN_MAP;
use super::KeySimulator;
use evdev::Key;

impl KeySimulator {
    pub fn execute_action(&self, action_id: &str) {
        log::debug!("[KeySimulator] Executing action: {}", action_id);
        if action_id == "none" {
            return;
        }

        if let Some(combo) = action_id.strip_prefix("custom:") {
            let mut keys = Vec::new();
            let layout = self.get_keyboard_layout();
            for part in combo.split('+') {
                let clean = part.trim().to_lowercase();
                if let Some(key) = super::key_map::get_key_by_layout(&clean, &layout) {
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
            "space_right" => Some(vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTALT, Key::KEY_RIGHT]),
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
