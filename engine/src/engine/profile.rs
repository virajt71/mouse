use crate::lock_ext::MutexExt;
use std::sync::atomic::Ordering;
use std::thread;

use super::app_change::compute_blocked_buttons;
use super::Engine;
use crate::config::Config;

impl Engine {
    pub fn refresh_active_profile(&self) {
        let (profile_name, mappings) = {
            let cfg = self.inner.config.lock_safe();
            let target = cfg.active_app_profile.clone();
            let mappings = cfg.get_resolved_mappings(&target);
            self.inner.cached_gesture_threshold.store(
                cfg.settings.gesture_threshold.max(0) as u32,
                Ordering::Relaxed,
            );
            self.inner.cached_gesture_deadzone.store(
                cfg.settings.gesture_deadzone.max(0) as u32,
                Ordering::Relaxed,
            );
            self.inner
                .cached_gesture_timeout_ms
                .store(cfg.settings.gesture_timeout_ms, Ordering::Relaxed);
            self.inner
                .cached_gesture_cooldown_ms
                .store(cfg.settings.gesture_cooldown_ms, Ordering::Relaxed);
            self.inner.cached_hscroll_threshold.store(
                cfg.settings.hscroll_threshold.max(0) as u32,
                Ordering::Relaxed,
            );
            (target, mappings)
        };

        *self.inner.current_profile.lock_safe() = profile_name.clone();
        *self.inner.active_profile_shared.lock_safe() = profile_name;
        {
            let mappings_arc: std::collections::HashMap<String, std::sync::Arc<str>> = mappings
                .iter()
                .map(|(k, v)| (k.clone(), std::sync::Arc::from(v.as_str())))
                .collect();
            *self.inner.active_mappings.write().unwrap() = mappings_arc;
        }

        let (blocked, hscroll_blocked) = compute_blocked_buttons(&mappings);

        *self.inner.blocked_buttons_arc.lock_safe() = blocked;
        self.inner
            .block_hscroll_arc
            .store(hscroll_blocked, Ordering::Relaxed);

        self.apply_keyboard_backlight();
    }

    pub fn select_profile(&self, name: &str) {
        log::info!("[Engine] Selecting active app profile: {}", name);
        {
            let mut cfg = self.inner.config.lock_safe();
            cfg.active_app_profile = name.to_string();
            let _ = cfg.save();
            self.increment_config_generation(&cfg);
        }
        self.refresh_active_profile();
    }

    pub fn add_profile(&self, name: &str) {
        log::info!("[Engine] Adding profile: {}", name);
        {
            let mut cfg = self.inner.config.lock_safe();
            let active_group = cfg.active_group.clone();
            if let Some(group) = cfg.profile_groups.get_mut(&active_group) {
                if !group.profiles.contains_key(name) {
                    let global_profile =
                        group.profiles.get("global").cloned().unwrap_or_else(|| {
                            crate::config::Profile {
                                label: name.to_string(),
                                apps: Vec::new(),
                                mappings: std::collections::HashMap::new(),
                                icon: String::new(),
                            }
                        });
                    let mut new_profile = global_profile;
                    new_profile.label = name.to_string();
                    new_profile.apps = Vec::new();
                    group.profiles.insert(name.to_string(), new_profile);
                    let _ = cfg.save();
                    self.increment_config_generation(&cfg);
                }
            }
        }
    }

    pub fn set_profile_icon(&self, name: &str, icon: &str) {
        if icon.is_empty() {
            return;
        }
        let mut cfg = self.inner.config.lock_safe();
        let active_group = cfg.active_group.clone();
        if let Some(group) = cfg.profile_groups.get_mut(&active_group) {
            if let Some(profile) = group.profiles.get_mut(name) {
                profile.icon = icon.to_string();
                let _ = cfg.save();
                self.increment_config_generation(&cfg);
            }
        }
    }

    pub fn delete_profile(&self, name: &str) {
        log::info!("[Engine] Deleting profile: {}", name);
        if name == "global" {
            return;
        }
        {
            let mut cfg = self.inner.config.lock_safe();
            let active_group = cfg.active_group.clone();
            if let Some(group) = cfg.profile_groups.get_mut(&active_group) {
                group.profiles.remove(name);
                if cfg.active_app_profile == name {
                    cfg.active_app_profile = "global".to_string();
                }
                let _ = cfg.save();
                self.increment_config_generation(&cfg);
            }
        }
        self.refresh_active_profile();
    }

    pub fn update_app_bindings(&self, profile_name: &str, app_bindings: &str) {
        log::info!(
            "[Engine] Updating app bindings for profile {}: {}",
            profile_name,
            app_bindings
        );
        let clean_exe = app_bindings
            .split(',')
            .next()
            .unwrap_or("")
            .trim()
            .to_lowercase();

        {
            let mut cfg = self.inner.config.lock_safe();
            let active_group = cfg.active_group.clone();
            if let Some(group) = cfg.profile_groups.get_mut(&active_group) {
                if !clean_exe.is_empty() {
                    let mut is_duplicate = false;
                    for (pname, pdata) in &group.profiles {
                        if pname != profile_name && pdata.apps.contains(&clean_exe) {
                            is_duplicate = true;
                            break;
                        }
                    }
                    if is_duplicate {
                        log::warn!("[Engine] Rejected mapping executable '{}' to profile '{}' because it is already mapped.", clean_exe, profile_name);
                        return;
                    }
                }

                if let Some(profile) = group.profiles.get_mut(profile_name) {
                    if clean_exe.is_empty() {
                        profile.apps = Vec::new();
                    } else {
                        profile.apps = vec![clean_exe];
                    }
                    let _ = cfg.save();
                    self.increment_config_generation(&cfg);
                }
            }
        }
    }

    pub fn update_profile_mappings(
        &self,
        profile_name: &str,
        mappings: std::collections::HashMap<String, String>,
    ) {
        log::info!("[Engine] Updating mappings for profile {}", profile_name);
        {
            let mut cfg = self.inner.config.lock_safe();
            let active_group = cfg.active_group.clone();
            if let Some(group) = cfg.profile_groups.get_mut(&active_group) {
                if let Some(profile) = group.profiles.get_mut(profile_name) {
                    for (k, v) in mappings {
                        profile.mappings.insert(k, v);
                    }
                    let _ = cfg.save();
                    self.increment_config_generation(&cfg);
                }
            }
        }
        log::info!("[Engine] Saved mappings, refreshing active profile...");
        self.refresh_active_profile();
    }

    pub fn apply_keyboard_backlight(&self) {
        let (backlight_effect, backlight_enabled) = {
            let cfg = self.inner.config.lock_safe();
            let active_profile = cfg.active_app_profile.clone();
            if let Some(profile) = cfg.get_profile(&active_profile) {
                let effect = profile.mappings.get("backlight_effect").cloned();
                let enabled = profile.mappings.get("backlight_enabled").cloned();
                (effect, enabled)
            } else {
                (None, None)
            }
        };

        log::debug!(
            "[Engine] Applying keyboard backlight settings: effect={:?}, enabled={:?}",
            backlight_effect,
            backlight_enabled
        );

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock_safe();
            for client in clients.iter_mut() {
                if client.is_connected() {
                    let layout = client.get_layout_key();
                    if layout.starts_with("mx_keys") || layout.starts_with("mx_mechanical") {
                        if let Some(ref enabled_str) = backlight_enabled {
                            let enabled = enabled_str == "true";
                            if let Err(e) = client.set_backlight_enabled(enabled) {
                                log::warn!(
                                    "[Engine] Failed to set backlight enabled to {} on '{}': {}",
                                    enabled,
                                    client.device_name,
                                    e
                                );
                            }
                        }
                        if let Some(ref effect) = backlight_effect {
                            if let Err(e) = client.set_backlight_effect(effect) {
                                log::warn!(
                                    "[Engine] Failed to set backlight effect to '{}' on '{}': {}",
                                    effect,
                                    client.device_name,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn cycle_backlight_effect(&self) {
        log::info!("[Engine] Cycle backlight effect triggered by Fn+Lightbulb");
        let active_profile = self.inner.active_profile_shared.lock_safe().clone();

        let mut new_effect = String::from("Static");
        let mut new_enabled = String::from("true");
        let mut mappings = std::collections::HashMap::new();

        {
            let cfg = self.inner.config.lock_safe();
            if let Some(profile) = cfg.get_profile(&active_profile) {
                let current_effect = profile
                    .mappings
                    .get("backlight_effect")
                    .map(|s| s.as_str())
                    .unwrap_or("Static");
                let current_enabled = profile
                    .mappings
                    .get("backlight_enabled")
                    .map(|s| s.as_str())
                    .unwrap_or("true");
                new_enabled = current_enabled.to_string();

                let effects = [
                    "Static",
                    "Contrast",
                    "Breathing",
                    "Waves",
                    "Reaction",
                    "Random",
                ];
                let current_idx = effects
                    .iter()
                    .position(|&x| x == current_effect)
                    .unwrap_or(0);
                let next_idx = (current_idx + 1) % effects.len();
                new_effect = effects[next_idx].to_string();

                mappings = profile.mappings.clone();
            }
        }

        mappings.insert("backlight_effect".to_string(), new_effect.clone());
        mappings.insert("backlight_enabled".to_string(), new_enabled.clone());

        // Update profile mappings and save/refresh/increment config generation
        self.update_profile_mappings(&active_profile, mappings);

        log::info!("[Engine] Cycled backlight effect to: {}", new_effect);
    }

    pub fn select_profile_group(&self, name: &str) {
        log::info!("[Engine] Selecting profile group: {}", name);
        {
            let mut cfg = self.inner.config.lock_safe();
            if cfg.profile_groups.contains_key(name) {
                cfg.active_group = name.to_string();
                cfg.active_app_profile = "global".to_string();
                let _ = cfg.save();
                self.increment_config_generation(&cfg);
            }
        }
        self.refresh_active_profile();
    }

    pub fn add_profile_group(&self, name: &str) {
        log::info!("[Engine] Adding profile group: {}", name);
        {
            let mut cfg = self.inner.config.lock_safe();
            if !cfg.profile_groups.contains_key(name) {
                let mut profiles = std::collections::HashMap::new();
                let default_mappings = cfg
                    .profile_groups
                    .get("default")
                    .and_then(|g| g.profiles.get("global"))
                    .map(|p| p.mappings.clone())
                    .unwrap_or_default();

                profiles.insert(
                    "global".to_string(),
                    crate::config::Profile {
                        label: "Default (All Apps)".to_string(),
                        apps: vec![],
                        mappings: default_mappings,
                        icon: String::new(),
                    },
                );
                cfg.profile_groups
                    .insert(name.to_string(), crate::config::ProfileGroup { profiles });
                let _ = cfg.save();
                self.increment_config_generation(&cfg);
            }
        }
    }

    pub fn delete_profile_group(&self, name: &str) {
        log::info!("[Engine] Deleting profile group: {}", name);
        if name == "default" {
            return;
        }
        {
            let mut cfg = self.inner.config.lock_safe();
            cfg.profile_groups.remove(name);
            if cfg.active_group == name {
                cfg.active_group = "default".to_string();
                cfg.active_app_profile = "global".to_string();
            }
            let _ = cfg.save();
            self.increment_config_generation(&cfg);
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
            let mut cfg = self.inner.config.lock_safe();
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
            self.increment_config_generation(&cfg);
        }

        self.inner
            .invert_vscroll_arc
            .store(invert_vscroll, Ordering::Relaxed);
        self.inner
            .invert_hscroll_arc
            .store(invert_hscroll, Ordering::Relaxed);
        self.inner
            .cached_gesture_threshold
            .store(gesture_threshold.max(0) as u32, Ordering::Relaxed);
        self.inner
            .cached_gesture_deadzone
            .store(gesture_deadzone.max(0) as u32, Ordering::Relaxed);
        self.inner
            .cached_hscroll_threshold
            .store(hscroll_threshold.max(0) as u32, Ordering::Relaxed);

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock_safe();
            for client in clients.iter_mut() {
                if client.is_connected()
                    && (client.get_layout_key().starts_with("mx_master")
                        || client.get_layout_key().starts_with("mx_anywhere"))
                {
                    let _ = client.set_dpi(dpi);
                    let _ = client.set_smart_shift(
                        &smart_shift_mode,
                        smart_shift_enabled,
                        smart_shift_threshold,
                    );
                }
            }
        });
    }

    pub fn update_keyboard_layout(&self, layout: &str) {
        log::info!("[Engine] Updating keyboard layout to: {}", layout);
        {
            let mut cfg = self.inner.config.lock_safe();
            cfg.settings.device_layout_overrides.insert(
                "keyboard_layout".to_string(),
                serde_json::Value::String(layout.to_string()),
            );
            let _ = cfg.save();
            self.increment_config_generation(&cfg);
        }
        self.inner.key_simulator.set_keyboard_layout(layout);
    }

    pub fn reload_config(&self) {
        log::info!("[Engine] Reloading config from disk");
        let (dpi, ss_mode, ss_enabled, ss_threshold, invert_hscroll, invert_vscroll, layout) = {
            let mut cfg = self.inner.config.lock_safe();
            *cfg = Config::load();
            self.increment_config_generation(&cfg);
            let layout = cfg
                .settings
                .device_layout_overrides
                .get("keyboard_layout")
                .and_then(|v| v.as_str())
                .unwrap_or("ANSI (US)")
                .to_string();
            (
                cfg.settings.dpi as u32,
                cfg.settings.smart_shift_mode.clone(),
                cfg.settings.smart_shift_enabled,
                cfg.settings.smart_shift_threshold as u8,
                cfg.settings.invert_hscroll,
                cfg.settings.invert_vscroll,
                layout,
            )
        };

        self.inner.key_simulator.set_keyboard_layout(&layout);

        self.inner
            .invert_vscroll_arc
            .store(invert_vscroll, Ordering::Relaxed);
        self.inner
            .invert_hscroll_arc
            .store(invert_hscroll, Ordering::Relaxed);

        self.refresh_active_profile();

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock_safe();
            for client in clients.iter_mut() {
                if client.is_connected()
                    && (client.get_layout_key().starts_with("mx_master")
                        || client.get_layout_key().starts_with("mx_anywhere"))
                {
                    let _ = client.set_dpi(dpi);
                    let _ = client.set_smart_shift(&ss_mode, ss_enabled, ss_threshold);
                }
            }
        });
    }
    pub fn apply_backlight_from_hid(&self, enabled: bool, effect_id: u8) {
        let effect = match effect_id {
            0x01 => "Static",
            0x02 => "Contrast",
            0x03 => "Breathing",
            0x04 => "Waves",
            0x05 => "Reaction",
            0x06 => "Random",
            _ => return, // unknown effect id, ignore
        };

        let profile_name = self.inner.active_profile_shared.lock_safe().clone();
        let mut cfg = self.inner.config.lock_safe();
        let active_group = cfg.active_group.clone();

        if let Some(group) = cfg.profile_groups.get_mut(&active_group) {
            if let Some(profile) = group.profiles.get_mut(&profile_name) {
                profile.mappings.insert(
                    "backlight_enabled".to_string(),
                    enabled.to_string(),
                );
                profile.mappings.insert(
                    "backlight_effect".to_string(),
                    effect.to_string(),
                );
                let _ = cfg.save();
                self.increment_config_generation(&cfg);
                log::info!(
                    "[Engine] Backlight synced from HID++: enabled={} effect={}",
                    enabled,
                    effect
                );
            }
        }
    }
}
