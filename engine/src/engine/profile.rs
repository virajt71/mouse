use std::sync::atomic::Ordering;
use std::thread;

use crate::config::Config;
use super::Engine;
use super::app_change::compute_blocked_buttons;

impl Engine {
    pub fn refresh_active_profile(&self) {
        let (profile_name, mappings) = {
            let cfg = self.inner.config.lock().unwrap();
            let target = cfg.active_app_profile.clone();
            let mappings = cfg.get_resolved_mappings(&target);
            self.inner.cached_gesture_threshold.store(cfg.settings.gesture_threshold.max(0) as u32, Ordering::Relaxed);
            self.inner.cached_gesture_deadzone.store(cfg.settings.gesture_deadzone.max(0) as u32, Ordering::Relaxed);
            self.inner.cached_gesture_timeout_ms.store(cfg.settings.gesture_timeout_ms, Ordering::Relaxed);
            self.inner.cached_gesture_cooldown_ms.store(cfg.settings.gesture_cooldown_ms, Ordering::Relaxed);
            (target, mappings)
        };

        *self.inner.current_profile.lock().unwrap() = profile_name.clone();
        *self.inner.active_profile_shared.lock().unwrap() = profile_name;
        *self.inner.active_mappings.lock().unwrap() = mappings.clone();

        let (blocked, hscroll_blocked) = compute_blocked_buttons(&mappings);

        *self.inner.blocked_buttons_arc.lock().unwrap() = blocked;
        self.inner.block_hscroll_arc.store(hscroll_blocked, Ordering::SeqCst);
    }

    pub fn select_profile(&self, name: &str) {
        log::info!("[Engine] Selecting active app profile: {}", name);
        {
            let mut cfg = self.inner.config.lock().unwrap();
            cfg.active_app_profile = name.to_string();
            let _ = cfg.save();
            self.increment_config_generation();
        }
        self.refresh_active_profile();
    }

    pub fn add_profile(&self, name: &str) {
        log::info!("[Engine] Adding profile: {}", name);
        {
            let mut cfg = self.inner.config.lock().unwrap();
            let active_group = cfg.active_group.clone();
            if let Some(group) = cfg.profile_groups.get_mut(&active_group) {
                if !group.profiles.contains_key(name) {
                    let global_profile = group.profiles.get("global").cloned().unwrap_or_else(|| {
                        crate::config::Profile {
                            label: name.to_string(),
                            apps: Vec::new(),
                            mappings: std::collections::HashMap::new(),
                        }
                    });
                    let mut new_profile = global_profile;
                    new_profile.label = name.to_string();
                    new_profile.apps = Vec::new();
                    group.profiles.insert(name.to_string(), new_profile);
                    let _ = cfg.save();
                    self.increment_config_generation();
                }
            }
        }
    }

    pub fn delete_profile(&self, name: &str) {
        log::info!("[Engine] Deleting profile: {}", name);
        if name == "global" {
            return;
        }
        {
            let mut cfg = self.inner.config.lock().unwrap();
            let active_group = cfg.active_group.clone();
            if let Some(group) = cfg.profile_groups.get_mut(&active_group) {
                group.profiles.remove(name);
                if cfg.active_app_profile == name {
                    cfg.active_app_profile = "global".to_string();
                }
                let _ = cfg.save();
                self.increment_config_generation();
            }
        }
        self.refresh_active_profile();
    }

    pub fn update_app_bindings(&self, profile_name: &str, app_bindings: &str) {
        log::info!("[Engine] Updating app bindings for profile {}: {}", profile_name, app_bindings);
        let apps: Vec<String> = app_bindings
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        {
            let mut cfg = self.inner.config.lock().unwrap();
            let active_group = cfg.active_group.clone();
            if let Some(group) = cfg.profile_groups.get_mut(&active_group) {
                if let Some(profile) = group.profiles.get_mut(profile_name) {
                    profile.apps = apps;
                    let _ = cfg.save();
                    self.increment_config_generation();
                }
            }
        }
    }

    pub fn update_profile_mappings(&self, profile_name: &str, mappings: std::collections::HashMap<String, String>) {
        log::info!("[Engine] Updating mappings for profile {}", profile_name);
        {
            let mut cfg = self.inner.config.lock().unwrap();
            let active_group = cfg.active_group.clone();
            if let Some(group) = cfg.profile_groups.get_mut(&active_group) {
                if let Some(profile) = group.profiles.get_mut(profile_name) {
                    for (k, v) in mappings {
                        profile.mappings.insert(k, v);
                    }
                    let _ = cfg.save();
                    self.increment_config_generation();
                }
            }
        }
        log::info!("[Engine] Saved mappings, refreshing active profile...");
        self.refresh_active_profile();
        log::info!("[Engine] Refreshed active profile, restarting keyboard hooks...");
        let _ = self.restart_keyboard_hooks();
        log::info!("[Engine] Keyboard hooks restarted successfully!");
    }

    pub fn select_profile_group(&self, name: &str) {
        log::info!("[Engine] Selecting profile group: {}", name);
        {
            let mut cfg = self.inner.config.lock().unwrap();
            if cfg.profile_groups.contains_key(name) {
                cfg.active_group = name.to_string();
                cfg.active_app_profile = "global".to_string();
                let _ = cfg.save();
                self.increment_config_generation();
            }
        }
        self.refresh_active_profile();
    }

    pub fn add_profile_group(&self, name: &str) {
        log::info!("[Engine] Adding profile group: {}", name);
        {
            let mut cfg = self.inner.config.lock().unwrap();
            if !cfg.profile_groups.contains_key(name) {
                let mut profiles = std::collections::HashMap::new();
                let default_mappings = cfg.profile_groups.get("default")
                    .and_then(|g| g.profiles.get("global"))
                    .map(|p| p.mappings.clone())
                    .unwrap_or_else(std::collections::HashMap::new);

                profiles.insert("global".to_string(), crate::config::Profile {
                    label: "Default (All Apps)".to_string(),
                    apps: vec![],
                    mappings: default_mappings,
                });
                cfg.profile_groups.insert(name.to_string(), crate::config::ProfileGroup { profiles });
                let _ = cfg.save();
                self.increment_config_generation();
            }
        }
    }

    pub fn delete_profile_group(&self, name: &str) {
        log::info!("[Engine] Deleting profile group: {}", name);
        if name == "default" {
            return;
        }
        {
            let mut cfg = self.inner.config.lock().unwrap();
            cfg.profile_groups.remove(name);
            if cfg.active_group == name {
                cfg.active_group = "default".to_string();
                cfg.active_app_profile = "global".to_string();
            }
            let _ = cfg.save();
            self.increment_config_generation();
        }
        self.refresh_active_profile();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_global_settings(
        &self,
        dpi: u32,
        smart_shift_mode: String,
        smart_shift_enabled: bool,
        smart_shift_threshold: u8,
        invert_hscroll: bool,
        invert_vscroll: bool,
        gesture_threshold: i32,
        gesture_deadzone: i32,
        accent_color: String,
        hscroll_threshold: i32,
    ) {
        log::info!(
            "[Engine] Updating global settings: DPI={}, SmartShift mode={}, enabled={}, threshold={}, invert_hscroll={}, invert_vscroll={}, gesture_threshold={}, gesture_deadzone={}, accent_color={}, hscroll_threshold={}",
            dpi, smart_shift_mode, smart_shift_enabled, smart_shift_threshold, invert_hscroll, invert_vscroll, gesture_threshold, gesture_deadzone, accent_color, hscroll_threshold
        );

        {
            let mut cfg = self.inner.config.lock().unwrap();
            cfg.settings.dpi = dpi as i32;
            cfg.settings.smart_shift_mode = smart_shift_mode.clone();
            cfg.settings.smart_shift_enabled = smart_shift_enabled;
            cfg.settings.smart_shift_threshold = smart_shift_threshold as i32;
            cfg.settings.invert_hscroll = invert_hscroll;
            cfg.settings.invert_vscroll = invert_vscroll;
            cfg.settings.gesture_threshold = gesture_threshold;
            cfg.settings.gesture_deadzone = gesture_deadzone;
            cfg.settings.accent_color = accent_color;
            cfg.settings.hscroll_threshold = hscroll_threshold;
            let _ = cfg.save();
            self.increment_config_generation();
        }

        self.inner.invert_vscroll_arc.store(invert_vscroll, Ordering::SeqCst);
        self.inner.invert_hscroll_arc.store(invert_hscroll, Ordering::SeqCst);
        self.inner.cached_gesture_threshold.store(gesture_threshold.max(0) as u32, Ordering::Relaxed);
        self.inner.cached_gesture_deadzone.store(gesture_deadzone.max(0) as u32, Ordering::Relaxed);

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock().unwrap();
            for client in clients.iter_mut() {
                if client.is_connected()
                    && (client.get_layout_key().starts_with("mx_master") || client.get_layout_key().starts_with("mx_anywhere")) {
                        let _ = client.set_dpi(dpi);
                        let _ = client.set_smart_shift(&smart_shift_mode, smart_shift_enabled, smart_shift_threshold);
                    }
            }
        });
    }

    pub fn reload_config(&self) {
        log::info!("[Engine] Reloading config from disk");
        let (dpi, ss_mode, ss_enabled, ss_threshold, invert_hscroll, invert_vscroll) = {
            let mut cfg = self.inner.config.lock().unwrap();
            *cfg = Config::load();
            self.increment_config_generation();
            (
                cfg.settings.dpi as u32,
                cfg.settings.smart_shift_mode.clone(),
                cfg.settings.smart_shift_enabled,
                cfg.settings.smart_shift_threshold as u8,
                cfg.settings.invert_hscroll,
                cfg.settings.invert_vscroll,
            )
        };

        self.inner.invert_vscroll_arc.store(invert_vscroll, Ordering::SeqCst);
        self.inner.invert_hscroll_arc.store(invert_hscroll, Ordering::SeqCst);

        self.refresh_active_profile();
        let _ = self.restart_keyboard_hooks();

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock().unwrap();
            for client in clients.iter_mut() {
                if client.is_connected()
                    && (client.get_layout_key().starts_with("mx_master") || client.get_layout_key().starts_with("mx_anywhere")) {
                        let _ = client.set_dpi(dpi);
                        let _ = client.set_smart_shift(&ss_mode, ss_enabled, ss_threshold);
                    }
            }
        });
    }
}
