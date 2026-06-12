use super::Engine;
use std::thread;

impl Engine {
    pub fn execute_engine_action(&self, action_id: &str) {
        if action_id == "none" {
            return;
        }

        if action_id == "toggle_smart_shift" {
            self.toggle_smart_shift();
        } else if action_id == "switch_scroll_mode" {
            self.switch_scroll_mode();
        } else if action_id == "cycle_dpi" {
            self.cycle_dpi();
        } else {
            self.inner.key_simulator.execute_action(action_id);
        }
    }

    pub fn toggle_smart_shift(&self) {
        let (mode, enabled, threshold) = {
            let mut cfg = self.inner.config.lock().unwrap();
            let next_enabled = !cfg.settings.smart_shift_enabled;
            cfg.settings.smart_shift_enabled = next_enabled;
            let _ = cfg.save();
            self.increment_config_generation();
            (
                cfg.settings.smart_shift_mode.clone(),
                next_enabled,
                cfg.settings.smart_shift_threshold as u8,
            )
        };
        log::info!("[Engine] Toggling SmartShift: enabled={}", enabled);

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock().unwrap();
            for client in clients.iter_mut() {
                if client.is_connected() && (client.get_layout_key().starts_with("mx_master")) {
                    if let Err(e) = client.set_smart_shift(&mode, enabled, threshold) {
                        log::error!(
                            "[Engine] Failed to set SmartShift on {}: {}",
                            client.device_name,
                            e
                        );
                    }
                }
            }
        });
    }

    pub fn switch_scroll_mode(&self) {
        let (mode, threshold) = {
            let mut cfg = self.inner.config.lock().unwrap();
            let next_mode = if cfg.settings.smart_shift_mode == "ratchet" {
                "freespin"
            } else {
                "ratchet"
            };
            cfg.settings.smart_shift_mode = next_mode.to_string();
            cfg.settings.smart_shift_enabled = false;
            let _ = cfg.save();
            self.increment_config_generation();
            (
                next_mode.to_string(),
                cfg.settings.smart_shift_threshold as u8,
            )
        };
        log::info!(
            "[Engine] Switching scroll mode to ratchet/freespin fixed: mode={}",
            mode
        );

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock().unwrap();
            for client in clients.iter_mut() {
                if client.is_connected() && (client.get_layout_key().starts_with("mx_master")) {
                    if let Err(e) = client.set_smart_shift(&mode, false, threshold) {
                        log::error!(
                            "[Engine] Failed to set scroll mode on {}: {}",
                            client.device_name,
                            e
                        );
                    }
                }
            }
        });
    }

    pub fn cycle_dpi(&self) {
        let presets = [800, 1200, 1600, 2400];
        let new_dpi = {
            let mut cfg = self.inner.config.lock().unwrap();
            let current = cfg.settings.dpi;
            let mut next_idx = 0;
            for (idx, &preset) in presets.iter().enumerate() {
                if preset == current {
                    next_idx = (idx + 1) % presets.len();
                    break;
                }
            }
            let val = presets[next_idx];
            cfg.settings.dpi = val;
            let _ = cfg.save();
            self.increment_config_generation();
            val
        };
        log::info!("[Engine] Cycling DPI to {}", new_dpi);

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock().unwrap();
            for client in clients.iter_mut() {
                if client.is_connected()
                    && (client.get_layout_key().starts_with("mx_master")
                        || client.get_layout_key().starts_with("mx_anywhere"))
                {
                    if let Err(e) = client.set_dpi(new_dpi as u32) {
                        log::error!(
                            "[Engine] Failed to write DPI to {}: {}",
                            client.device_name,
                            e
                        );
                    }
                }
            }
        });
    }
}
