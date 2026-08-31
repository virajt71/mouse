use super::Engine;
use crate::lock_ext::MutexExt;
use std::thread;

impl Engine {
    pub fn execute_engine_action(&self, action_id: &str) {
        if action_id == "none" {
            return;
        }

        // Smart Action: run a named multi-step macro from settings.
        if let Some(macro_name) = action_id.strip_prefix("macro:") {
            let steps = {
                let cfg = self.inner.config.lock_safe();
                cfg.settings.macros.get(macro_name).cloned()
            };
            if let Some(steps) = steps {
                for step in steps {
                    self.execute_engine_action(&step);
                }
            } else {
                log::warn!("[Engine] Unknown macro referenced by ring: {macro_name}");
            }
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

    /// Open the Actions Ring overlay in the GUI with the layout for the
    /// currently focused application. Bails out if the ring is disabled or no
    /// GUI broadcast handle is wired in.
    pub fn open_actions_ring(&self) {
        let (enabled, layout_json) = {
            let cfg = self.inner.config.lock_safe();
            if !cfg.settings.actions_ring.enabled {
                return;
            }
            let app = self.inner.last_detected_exe.lock_safe().clone();
            let app_id = if app.is_empty() {
                None
            } else {
                Some(app.as_str())
            };
            let layout = cfg.settings.actions_ring.effective_layout(app_id);
            let json = serde_json::to_string(&layout).unwrap_or_default();
            (true, json)
        };
        if !enabled {
            return;
        }
        let session_id = self
            .inner
            .actions_ring_session
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        let folders_json = {
            let cfg = self.inner.config.lock_safe();
            serde_json::to_string(&cfg.settings.actions_ring.folders).unwrap_or_default()
        };
        if let Some(bc) = self.inner.actions_ring_bc.lock_safe().as_ref() {
            crate::grpc::broadcast_actions_ring(true, session_id, &layout_json, &folders_json, bc);
        }
    }

    pub fn toggle_smart_shift(&self) {
        let (mode, enabled, threshold, cfg_snapshot) = {
            let mut cfg = self.inner.config.lock_safe();
            let next_enabled = !cfg.settings.smart_shift_enabled;
            cfg.settings.smart_shift_enabled = next_enabled;
            let _ = cfg.save();
            (
                cfg.settings.smart_shift_mode.clone(),
                next_enabled,
                cfg.settings.smart_shift_threshold as u8,
                cfg.clone(),
            )
        };
        self.increment_config_generation(&cfg_snapshot);
        log::info!("[Engine] Toggling SmartShift: enabled={}", enabled);

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock_safe();
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
        let (mode, threshold, cfg_snapshot) = {
            let mut cfg = self.inner.config.lock_safe();
            let next_mode = if cfg.settings.smart_shift_mode == "ratchet" {
                "freespin"
            } else {
                "ratchet"
            };
            cfg.settings.smart_shift_mode = next_mode.to_string();
            cfg.settings.smart_shift_enabled = false;
            let _ = cfg.save();
            (
                next_mode.to_string(),
                cfg.settings.smart_shift_threshold as u8,
                cfg.clone(),
            )
        };
        self.increment_config_generation(&cfg_snapshot);
        log::info!(
            "[Engine] Switching scroll mode to ratchet/freespin fixed: mode={}",
            mode
        );

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock_safe();
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
        let (new_dpi, cfg_snapshot) = {
            let mut cfg = self.inner.config.lock_safe();
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
            (val, cfg.clone())
        };
        self.increment_config_generation(&cfg_snapshot);
        log::info!("[Engine] Cycling DPI to {}", new_dpi);

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock_safe();
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
