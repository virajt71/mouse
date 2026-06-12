use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use super::Engine;
use crate::hidpp::HidppClient;
use crate::input::{KeyboardHook, MouseHook};

lazy_static::lazy_static! {
    static ref CONNECTING_PATHS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

impl Engine {
    pub fn restart_keyboard_hooks(&self) -> anyhow::Result<()> {
        let mappings = Arc::new(std::sync::Mutex::new(
            self.inner.config.lock().unwrap().get_active_mappings(),
        ));
        let mut kb_hooks = self.inner.keyboard_hooks.lock().unwrap();
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
                )
                .is_ok()
            {
                kb_hooks.push(hook);
            }
        }
        Ok(())
    }

    pub(crate) fn spawn_hidpp_thread(&self) -> anyhow::Result<()> {
        let inner = self.inner.clone();
        let engine_clone = self.clone();
        thread::Builder::new()
            .name("HidppThread".to_string())
            .spawn(move || {
                let mut last_reconnect = Instant::now() - Duration::from_secs(5);
                let mut last_mouse_check = Instant::now() - Duration::from_secs(5);
                let mut last_mouse_attempt_path: Option<String> = None;
                let mut last_mouse_attempt_time: Option<Instant> = Some(Instant::now());
                let mut failed_keyboards: std::collections::HashMap<String, Instant> = std::collections::HashMap::new();
                while inner.running.load(Ordering::SeqCst) {
                    if last_mouse_check.elapsed() >= Duration::from_millis(500) {
                        last_mouse_check = Instant::now();
                        let is_hook_running = {
                            let hook_lock = inner.mouse_hook.lock().unwrap();
                            if let Some(hook) = &*hook_lock {
                                hook.is_running()
                            } else {
                                false
                            }
                        };

                        if !is_hook_running {
                            crate::flow::FLOW_MANAGER.set_active_peer(None);
                            let hook_is_none = inner.mouse_hook.lock().unwrap().is_none();

                            if let Some(mouse_path) = crate::input::find_logitech_mouse() {
                                let should_attempt = match (&last_mouse_attempt_path, last_mouse_attempt_time) {
                                    (Some(last_path), Some(last_time)) if *last_path == mouse_path => {
                                        last_time.elapsed() >= Duration::from_secs(2)
                                    }
                                    _ => true,
                                };

                                if should_attempt {
                                    last_mouse_attempt_path = Some(mouse_path.clone());
                                    last_mouse_attempt_time = Some(Instant::now());

                                    log::info!("[Engine] Mouse hook not running but mouse detected — attempting to {}start...",
                                        if hook_is_none { "" } else { "re" });

                                    if let Some(mut hook) = inner.mouse_hook.lock().unwrap().take() {
                                        hook.stop();
                                    }

                                    let engine = engine_clone.clone();
                                    let mut hook = MouseHook::new(move |event| {
                                        engine.handle_mouse_hook_event(event);
                                    });

                                    match hook.start(
                                        inner.blocked_buttons_arc.clone(),
                                        inner.invert_vscroll_arc.clone(),
                                        inner.invert_hscroll_arc.clone(),
                                        inner.block_hscroll_arc.clone(),
                                        inner.gesture_active_arc.clone(),
                                        inner.key_simulator.device(),
                                    ) {
                                        Ok(()) => {
                                            log::info!("[Engine] Mouse hook started successfully.");
                                            *inner.mouse_hook.lock().unwrap() = Some(hook);
                                            last_mouse_attempt_path = None;
                                            last_mouse_attempt_time = None;
                                        }
                                        Err(e) => {
                                            log::warn!("[Engine] Mouse hook start failed for {}: {}", mouse_path, e);
                                        }
                                    }
                                }
                            }
                        }

                        // Check keyboard hooks individually
                        let kb_paths = crate::input::find_logitech_keyboards();
                        let mut kb_hooks = inner.keyboard_hooks.lock().unwrap();

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
                            let has_running_hook = kb_hooks.iter().any(|h| {
                                h.device_path() == Some(path) && h.is_running()
                            });

                            if !has_running_hook {
                                let should_attempt = match failed_keyboards.get(path) {
                                    Some(last_time) => last_time.elapsed() >= Duration::from_secs(2),
                                    None => true,
                                };

                                if should_attempt {
                                    failed_keyboards.insert(path.clone(), Instant::now());
                                    log::info!("[Engine] Keyboard hook not running for {} — attempting to start...", path);

                                    // Remove any existing dead hook for this path
                                    kb_hooks.retain(|h| h.device_path() != Some(path));

                                    let mappings = Arc::new(std::sync::Mutex::new(
                                        inner.config.lock().unwrap().get_active_mappings()
                                    ));
                                    let mut hook = KeyboardHook::new();
                                    match hook.start(
                                        path.clone(),
                                        mappings,
                                        inner.key_simulator.device(),
                                        inner.key_simulator.clone(),
                                        engine_clone.clone(),
                                    ) {
                                        Ok(()) => {
                                            log::info!("[Engine] Keyboard hook started successfully for {}.", path);
                                            kb_hooks.push(hook);
                                            failed_keyboards.remove(path);
                                        }
                                        Err(e) => {
                                            let error_str = e.to_string();
                                            if error_str.contains("No such file") || error_str.contains("No such device") {
                                                log::info!("[Engine] Keyboard hook start bypassed for {} (device disconnected/disconnecting)", path);
                                            } else {
                                                log::warn!("[Engine] Keyboard hook start failed for {}: {}", path, e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if last_reconnect.elapsed() >= Duration::from_secs(1) {
                        last_reconnect = Instant::now();
                        let mut api_lock = inner.hid_api.lock().unwrap();
                        if api_lock.is_none() {
                            *api_lock = hidapi::HidApi::new().ok();
                        }

                        let api = match api_lock.as_mut() {
                            Some(a) => {
                                let _ = a.refresh_devices();
                                a
                            },
                            None => continue,
                        };

                        let dummy_client = HidppClient::new();
                        let available = dummy_client.list_hidpp_devices(api);

                        let mut clients = inner.hid_clients.lock().unwrap();
                        clients.retain(|c| c.is_connected());

                        let mut connecting = CONNECTING_PATHS.lock().unwrap();

                        for info in available {
                            let path_str = info.path().to_string_lossy().to_string();
                            let already_opened = clients.iter().any(|c| c.device_path == path_str);
                            let is_connecting = connecting.contains(&path_str);

                            if !already_opened && !is_connecting {
                                connecting.insert(path_str.clone());

                                let inner_clone = inner.clone();
                                let engine_clone = engine_clone.clone();
                                let path_cstring = info.path().to_owned();
                                let path_str_clone = path_str.clone();

                                std::thread::spawn(move || {
                                    let api_temp = match hidapi::HidApi::new() {
                                        Ok(a) => a,
                                        Err(_) => {
                                            CONNECTING_PATHS.lock().unwrap().remove(&path_str_clone);
                                            return;
                                        }
                                    };

                                    let mut new_client = HidppClient::new();
                                    if new_client.open_path(&api_temp, &path_cstring).is_ok() {
                                        log::info!("[Engine] HID++ device connected: {}", new_client.device_name);
                                        let layout = new_client.get_layout_key();
                                        let mut new_keyboard_found = false;
                                        if layout.starts_with("mx_keys") || layout.starts_with("mx_mechanical") {
                                            new_keyboard_found = true;
                                        }
                                        if layout.starts_with("mx_master") || layout.starts_with("mx_anywhere") {
                                            let dpi = {
                                                let cfg = inner_clone.config.lock().unwrap();
                                                cfg.settings.dpi as u32
                                             };
                                            let (ss_mode, ss_enabled, ss_threshold) = {
                                                let cfg = inner_clone.config.lock().unwrap();
                                                (
                                                    cfg.settings.smart_shift_mode.clone(),
                                                    cfg.settings.smart_shift_enabled,
                                                    cfg.settings.smart_shift_threshold as u8,
                                                )
                                            };
                                            let _ = new_client.set_dpi(dpi);
                                            let _ = new_client.set_smart_shift(&ss_mode, ss_enabled, ss_threshold);
                                        }

                                        {
                                            let mut clients = inner_clone.hid_clients.lock().unwrap();
                                            clients.push(new_client);
                                        }

                                        if new_keyboard_found {
                                            let _ = engine_clone.restart_keyboard_hooks();
                                        }
                                    }

                                    CONNECTING_PATHS.lock().unwrap().remove(&path_str_clone);
                                });
                            }
                        }
                    }

                    let mut all_events = Vec::new();
                    let has_clients = {
                        let mut clients = inner.hid_clients.lock().unwrap();
                        for client in clients.iter_mut() {
                            if let Ok(evs) = client.poll_events() {
                                for ev in evs {
                                    all_events.push(ev);
                                }
                            }
                        }
                        !clients.is_empty()
                    };

                    for ev in all_events {
                        engine_clone.handle_hid_event(ev);
                    }

                    if has_clients {
                        if inner.gesture_active_arc.load(Ordering::Relaxed) {
                            thread::sleep(Duration::from_millis(5));
                        } else {
                            thread::sleep(Duration::from_millis(20));
                        }
                    } else {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            })?;

        Ok(())
    }
}
