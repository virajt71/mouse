use crate::lock_ext::MutexExt;
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use super::Engine;
use crate::hidpp::HidppClient;
use crate::input::{KeyboardHook, MouseHook};

static CONNECTING_PATHS: std::sync::LazyLock<Mutex<HashSet<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashSet::new()));

impl Engine {
    pub fn restart_keyboard_hooks(&self) -> anyhow::Result<()> {
        let mappings = self.inner.active_mappings.clone();
        let mut kb_hooks = self.inner.keyboard_hooks.lock_safe();
        for hook in kb_hooks.iter_mut() {
            hook.stop();
        }
        kb_hooks.clear();
        let kb_paths = crate::input::find_logitech_keyboards();
        for path in kb_paths {
            let mut hook = KeyboardHook::new();
            if hook
                .start(
                    path,
                    mappings.clone(),
                    self.inner.key_simulator.device(),
                    self.inner.key_simulator.clone(),
                    self.clone(),
                    self.inner.modifier_state.clone(),
                )
                .is_ok()
            {
                kb_hooks.push(hook);
            }
        }
        Ok(())
    }

    fn check_mouse_hook(&self, failed_mice: &mut std::collections::HashMap<String, Instant>) {
        let inner = &self.inner;
        let mouse_paths = crate::input::find_logitech_mice();
        let mut mouse_hooks = inner.mouse_hooks.lock_safe();

        // 1. Remove hooks that are no longer physically connected
        mouse_hooks.retain(|hook| {
            if let Some(path) = hook.device_path() {
                if mouse_paths.contains(&path.to_string()) {
                    true
                } else {
                    log::info!("[Engine] Mouse {} disconnected - stopping hook.", path);
                    false
                }
            } else {
                false
            }
        });

        if mouse_hooks.is_empty() {
            crate::flow::FLOW_MANAGER.set_active_peer(None);
        }

        // 2. Restart/Start hooks for all detected Logitech mice, with 2s cooldown
        for path in &mouse_paths {
            let has_running_hook = mouse_hooks
                .iter()
                .any(|h| h.device_path() == Some(path) && h.is_running());

            if !has_running_hook {
                let should_attempt = match failed_mice.get(path) {
                    Some(last_time) => last_time.elapsed() >= Duration::from_secs(2),
                    None => true,
                };

                if should_attempt {
                    failed_mice.insert(path.clone(), Instant::now());
                    log::info!(
                        "[Engine] Mouse hook not running for {} — attempting to start...",
                        path
                    );

                    // Remove any existing dead hook for this path
                    mouse_hooks.retain(|h| h.device_path() != Some(path));

                    let engine = self.clone();
                    let mut hook = MouseHook::new(move |event| {
                        engine.handle_mouse_hook_event(event);
                    });

                    match hook.start(
                        path.clone(),
                        inner.blocked_buttons_arc.clone(),
                        inner.invert_vscroll_arc.clone(),
                        inner.invert_hscroll_arc.clone(),
                        inner.block_hscroll_arc.clone(),
                        inner.gesture_active_arc.clone(),
                        inner.key_simulator.device(),
                    ) {
                        Ok(()) => {
                            log::info!("[Engine] Mouse hook started successfully for {}.", path);
                            mouse_hooks.push(hook);
                            failed_mice.remove(path);
                        }
                        Err(e) => {
                            let error_str = e.to_string();
                            if error_str.contains("No such file")
                                || error_str.contains("No such device")
                            {
                                log::info!(
                                    "[Engine] Mouse hook start bypassed for {} (device disconnected/disconnecting)",
                                    path
                                );
                            } else {
                                log::warn!("[Engine] Mouse hook start failed for {}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_keyboard_hooks(
        &self,
        failed_keyboards: &mut std::collections::HashMap<String, Instant>,
    ) {
        let inner = &self.inner;
        let kb_paths = crate::input::find_logitech_keyboards();
        let mut kb_hooks = inner.keyboard_hooks.lock_safe();

        // 1. Remove hooks that are no longer physically connected
        kb_hooks.retain(|hook| {
            if let Some(path) = hook.device_path() {
                if kb_paths.contains(&path.to_string()) {
                    true
                } else {
                    log::info!("[Engine] Keyboard {} disconnected - stopping hook.", path);
                    false
                }
            } else {
                false
            }
        });

        // 2. Restart hooks that are dead, with 2s cooldown
        for path in &kb_paths {
            let has_running_hook = kb_hooks
                .iter()
                .any(|h| h.device_path() == Some(path) && h.is_running());

            if !has_running_hook {
                let should_attempt = match failed_keyboards.get(path) {
                    Some(last_time) => last_time.elapsed() >= Duration::from_secs(2),
                    None => true,
                };

                if should_attempt {
                    failed_keyboards.insert(path.clone(), Instant::now());
                    log::info!(
                        "[Engine] Keyboard hook not running for {} — attempting to start...",
                        path
                    );

                    // Remove any existing dead hook for this path
                    kb_hooks.retain(|h| h.device_path() != Some(path));

                    let mappings = inner.active_mappings.clone();
                    let mut hook = KeyboardHook::new();
                    match hook.start(
                        path.clone(),
                        mappings,
                        inner.key_simulator.device(),
                        inner.key_simulator.clone(),
                        self.clone(),
                        inner.modifier_state.clone(),
                    ) {
                        Ok(()) => {
                            log::info!("[Engine] Keyboard hook started successfully for {}.", path);
                            kb_hooks.push(hook);
                            failed_keyboards.remove(path);
                        }
                        Err(e) => {
                            let error_str = e.to_string();
                            if error_str.contains("No such file")
                                || error_str.contains("No such device")
                            {
                                log::info!(
                                    "[Engine] Keyboard hook start bypassed for {} (device disconnected/disconnecting)",
                                    path
                                );
                            } else {
                                log::warn!(
                                    "[Engine] Keyboard hook start failed for {}: {}",
                                    path,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_hid_hotplug(&self) {
        /// RAII guard: removes `path` from `CONNECTING_PATHS` when dropped.
        /// Guarantees cleanup on any return path — including future early returns.
        struct ConnGuard(String);
        impl Drop for ConnGuard {
            fn drop(&mut self) {
                CONNECTING_PATHS.lock_safe().remove(&self.0);
            }
        }

        let inner = &self.inner;
        let mut api_lock = inner.hid_api.lock_safe();
        if api_lock.is_none() {
            *api_lock = hidapi::HidApi::new().ok();
        }

        let api = match api_lock.as_mut() {
            Some(a) => {
                let _ = a.refresh_devices();
                a
            }
            None => return,
        };

        let dummy_client = HidppClient::new();
        let available = dummy_client.list_hidpp_devices(api);

        let mut clients = inner.hid_clients.lock_safe();
        let old_len = clients.len();
        clients.retain(|c| c.is_connected());
        let new_len = clients.len();
        let changed = old_len != new_len;
        let opened_paths: std::collections::HashSet<String> =
            clients.iter().map(|c| c.device_path.clone()).collect();
        drop(clients);
        if changed {
            self.update_cached_device_state();
        }

        let mut connecting = CONNECTING_PATHS.lock_safe();

        for info in available {
            let path_str = info.path().to_string_lossy().to_string();
            let already_opened = opened_paths.contains(&path_str);
            let is_connecting = connecting.contains(&path_str);

            if !already_opened && !is_connecting {
                connecting.insert(path_str.clone());

                let inner_clone = inner.clone();
                let engine_clone = self.clone();
                let path_cstring = info.path().to_owned();
                let path_str_clone = path_str.clone();

                std::thread::spawn(move || {
                    // Guard guarantees removal from CONNECTING_PATHS on any return path.
                    let _guard = ConnGuard(path_str_clone.clone());

                    let api_temp = match hidapi::HidApi::new() {
                        Ok(a) => a,
                        Err(_) => return,
                    };

                    let mut new_client = HidppClient::new();
                    if new_client.open_path(&api_temp, &path_cstring).is_ok() {
                        log::info!(
                            "[Engine] HID++ device connected: {}",
                            new_client.device_name
                        );
                        let layout = new_client.get_layout_key();
                        let mut new_keyboard_found = false;
                        if layout.starts_with("mx_keys") || layout.starts_with("mx_mechanical") {
                            new_keyboard_found = true;
                        }
                        if layout.starts_with("mx_master") || layout.starts_with("mx_anywhere") {
                            let dpi = {
                                let cfg = inner_clone.config.lock_safe();
                                cfg.settings.dpi as u32
                            };
                            let (ss_mode, ss_enabled, ss_threshold) = {
                                let cfg = inner_clone.config.lock_safe();
                                (
                                    cfg.settings.smart_shift_mode.clone(),
                                    cfg.settings.smart_shift_enabled,
                                    cfg.settings.smart_shift_threshold as u8,
                                )
                            };
                            let _ = new_client.set_dpi(dpi);
                            let _ = new_client.set_smart_shift(&ss_mode, ss_enabled, ss_threshold);
                        }

                        if let Some(active_ch) = new_client.active_host_channel() {
                            let cfg_opt = {
                                let mut cfg = inner_clone.config.lock_safe();
                                if cfg.settings.flow_local_channel_index != active_ch {
                                    cfg.settings.flow_local_channel_index = active_ch;
                                    let _ = cfg.save();
                                    Some(cfg.clone())
                                } else {
                                    None
                                }
                            };
                            if let Some(cfg) = cfg_opt {
                                engine_clone.increment_config_generation(&cfg);
                            }
                        }

                        {
                            let mut clients = inner_clone.hid_clients.lock_safe();
                            clients.push(new_client);
                        }
                        engine_clone.update_cached_device_state();

                        if new_keyboard_found {
                            let _ = engine_clone.restart_keyboard_hooks();
                            engine_clone.apply_keyboard_backlight();
                        }
                    }
                    // _guard drops here → path removed from CONNECTING_PATHS
                });
            }
        }
    }

    pub(crate) fn spawn_hidpp_thread(&self) -> anyhow::Result<()> {
        let inner = self.inner.clone();
        let engine_clone = self.clone();
        thread::Builder::new()
            .name("HidppThread".to_string())
            .spawn(move || {
                let mut last_reconnect = Instant::now() - Duration::from_secs(5);
                let mut last_mouse_check = Instant::now() - Duration::from_secs(5);
                let mut failed_mice: std::collections::HashMap<String, Instant> =
                    std::collections::HashMap::new();
                let mut failed_keyboards: std::collections::HashMap<String, Instant> =
                    std::collections::HashMap::new();
                while inner.running.load(Ordering::Acquire) {
                    if last_mouse_check.elapsed() >= Duration::from_millis(500) {
                        last_mouse_check = Instant::now();
                        engine_clone.check_mouse_hook(&mut failed_mice);
                        engine_clone.check_keyboard_hooks(&mut failed_keyboards);
                    }

                    if last_reconnect.elapsed() >= Duration::from_secs(1) {
                        last_reconnect = Instant::now();
                        engine_clone.check_hid_hotplug();
                    }

                    let mut all_events = Vec::new();
                    let num_clients = { inner.hid_clients.lock_safe().len() };

                    for idx in 0..num_clients {
                        let mut client_evs = Vec::new();
                        {
                            let mut clients = inner.hid_clients.lock_safe();
                            if let Some(client) = clients.get_mut(idx) {
                                if let Ok(evs) = client.poll_events() {
                                    client_evs = evs;
                                }
                            }
                        }
                        for ev in client_evs {
                            all_events.push(ev);
                        }
                        // Short sleep to allow UI thread to acquire clients lock if needed
                        thread::sleep(Duration::from_millis(2));
                    }

                    for ev in all_events {
                        engine_clone.handle_hid_event(ev);
                    }

                    if num_clients > 0 {
                        thread::sleep(Duration::from_millis(10));
                    } else {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            })?;

        Ok(())
    }
}
