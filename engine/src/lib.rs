#![allow(dead_code)]
pub mod config;
pub mod hidpp;
pub mod mouse_hook;
pub mod keyboard_hook;
pub mod key_simulator;
pub mod app_detector;
pub mod bluetooth;
pub mod battery;
pub mod receiver;
pub mod cache;
pub mod worker;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use evdev::Key;

use self::config::Config;
use self::key_simulator::{KeySimulator, is_mouse_button_action, get_mouse_button_key};
use self::app_detector::AppDetector;
use self::hidpp::{HidppClient, HidppEvent};
use self::mouse_hook::{MouseHook, MouseHookEvent};
use self::keyboard_hook::KeyboardHook;

struct GestureState {
    delta_x: f32,
    delta_y: f32,
    last_move_at: Instant,
    cooldown_until: Instant,
    input_source: Option<String>,
    button: Option<String>,
}

struct EngineInner {
    config: Mutex<Config>,
    config_generation: AtomicU64,
    key_simulator: KeySimulator,
    app_detector: Mutex<Option<AppDetector>>,
    mouse_hook: Mutex<Option<MouseHook>>,
    keyboard_hooks: Mutex<Vec<KeyboardHook>>,
    hid_api: Mutex<Option<hidapi::HidApi>>,
    hid_clients: Mutex<Vec<HidppClient>>,
    selected_device_idx: Mutex<usize>,
    running: AtomicBool,
    current_profile: Mutex<String>,
    active_mappings: Mutex<HashMap<String, String>>,
    active_profile_shared: Arc<Mutex<String>>,
    last_detected_exe: Mutex<String>,

    // Shared state variables with MouseHook
    blocked_buttons_arc: Arc<Mutex<Vec<Key>>>,
    invert_vscroll_arc: Arc<AtomicBool>,
    invert_hscroll_arc: Arc<AtomicBool>,
    block_hscroll_arc: Arc<AtomicBool>,
    gesture_active_arc: Arc<AtomicBool>,

    // Gesture tracking state
    gesture_tracking: AtomicBool,
    gesture_triggered: AtomicBool,
    gesture_state: Mutex<GestureState>,

    // HScroll tracking state
    hscroll_accum_left: Mutex<f32>,
    hscroll_accum_right: Mutex<f32>,
    hscroll_last_fire_left: Mutex<Instant>,
    hscroll_last_fire_right: Mutex<Instant>,

    // Cached gesture settings (atomics for lock-free hot-path reads)
    cached_gesture_threshold: AtomicU32,   // f32 bits
    cached_gesture_deadzone: AtomicU32,    // f32 bits
    cached_gesture_timeout_ms: AtomicU64,
    cached_gesture_cooldown_ms: AtomicU64,
}

#[derive(Clone)]
pub struct Engine {
    inner: Arc<EngineInner>,
}

fn compute_blocked_buttons(mappings: &std::collections::HashMap<String, String>) -> (Vec<Key>, bool) {
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

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        let config = Config::load();
        let key_simulator = KeySimulator::new();
        let current_profile = config.active_profile.clone();

        let invert_vscroll = config.settings.invert_vscroll;
        let invert_hscroll = config.settings.invert_hscroll;
        let init_gesture_threshold = config.settings.gesture_threshold.max(0) as u32;
        let init_gesture_deadzone = config.settings.gesture_deadzone.max(0) as u32;
        let init_gesture_timeout_ms = config.settings.gesture_timeout_ms;
        let init_gesture_cooldown_ms = config.settings.gesture_cooldown_ms;

        let inner = EngineInner {
            config: Mutex::new(config),
            config_generation: AtomicU64::new(1),
            key_simulator,
            app_detector: Mutex::new(None),
            mouse_hook: Mutex::new(None),
            keyboard_hooks: Mutex::new(Vec::new()),
            hid_api: Mutex::new(hidapi::HidApi::new().ok()),
            hid_clients: Mutex::new(Vec::new()),
            selected_device_idx: Mutex::new(0),
            running: AtomicBool::new(false),
            current_profile: Mutex::new(current_profile.clone()),
            active_mappings: Mutex::new(HashMap::new()),
            active_profile_shared: Arc::new(Mutex::new(current_profile)),
            last_detected_exe: Mutex::new(String::new()),

            blocked_buttons_arc: Arc::new(Mutex::new(Vec::new())),
            invert_vscroll_arc: Arc::new(AtomicBool::new(invert_vscroll)),
            invert_hscroll_arc: Arc::new(AtomicBool::new(invert_hscroll)),
            block_hscroll_arc: Arc::new(AtomicBool::new(false)),
            gesture_active_arc: Arc::new(AtomicBool::new(false)),

            gesture_tracking: AtomicBool::new(false),
            gesture_triggered: AtomicBool::new(false),
            gesture_state: Mutex::new(GestureState {
                delta_x: 0.0,
                delta_y: 0.0,
                last_move_at: Instant::now(),
                cooldown_until: Instant::now(),
                input_source: None,
                button: None,
            }),

            hscroll_accum_left: Mutex::new(0.0),
            hscroll_accum_right: Mutex::new(0.0),
            hscroll_last_fire_left: Mutex::new(Instant::now() - Duration::from_secs(1)),
            hscroll_last_fire_right: Mutex::new(Instant::now() - Duration::from_secs(1)),

            cached_gesture_threshold: AtomicU32::new(init_gesture_threshold),
            cached_gesture_deadzone: AtomicU32::new(init_gesture_deadzone),
            cached_gesture_timeout_ms: AtomicU64::new(init_gesture_timeout_ms),
            cached_gesture_cooldown_ms: AtomicU64::new(init_gesture_cooldown_ms),
        };

        let engine = Engine {
            inner: Arc::new(inner),
        };
        engine.refresh_active_profile();
        engine
    }

    fn refresh_active_profile(&self) {
        let (profile_name, mappings) = {
            let cfg = self.inner.config.lock().unwrap();
            let target = cfg.active_profile.clone();
            let mappings = cfg.get_resolved_mappings(&target);
            // Also refresh cached gesture settings atomics
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

    pub fn start(&self) -> anyhow::Result<()> {
        if self.inner.running.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.inner.running.store(true, Ordering::SeqCst);

        // Ensure virtual uinput device is created eagerly before mouse hook starts
        self.inner.key_simulator.ensure_device();

        // Reset tracking states
        self.inner.gesture_active_arc.store(false, Ordering::SeqCst);
        self.inner.gesture_tracking.store(false, Ordering::SeqCst);
        self.inner.gesture_triggered.store(false, Ordering::SeqCst);

        // Apply profile initial bindings
        self.refresh_active_profile();

        // Initial Keyboard Hook start
        let _ = self.restart_keyboard_hooks();

        // Start MouseHook — non-fatal: if no mouse is present at launch,
        // the HidppThread hot-plug checker will retry every 500ms.
        {
            let mut hook_lock = self.inner.mouse_hook.lock().unwrap();
            if hook_lock.is_none() {
                let engine = self.clone();
                let mut hook = MouseHook::new(move |event| {
                    engine.handle_mouse_hook_event(event);
                });

                match hook.start(
                    self.inner.blocked_buttons_arc.clone(),
                    self.inner.invert_vscroll_arc.clone(),
                    self.inner.invert_hscroll_arc.clone(),
                    self.inner.block_hscroll_arc.clone(),
                    self.inner.gesture_active_arc.clone(),
                    self.inner.key_simulator.device(),
                ) {
                    Ok(()) => {
                        *hook_lock = Some(hook);
                    }
                    Err(e) => {
                        log::warn!("[Engine] Mouse hook not started at launch ({}). Will retry when device is connected.", e);
                        // hook_lock stays None — hot-plug loop will start it later
                    }
                }
            }
        }

        // Start AppDetector
        let mut app_det_lock = self.inner.app_detector.lock().unwrap();
        if app_det_lock.is_none() {
            let engine = self.clone();
            let mut app_det = AppDetector::new(move |exe_name| {
                engine.handle_app_change(exe_name);
            });
            app_det.start();
            *app_det_lock = Some(app_det);
        }

        let inner = self.inner.clone();
        let engine_clone = self.clone();
        thread::Builder::new()
            .name("HidppThread".to_string())
            .spawn(move || {
                let mut last_reconnect = Instant::now() - Duration::from_secs(5);
                let mut last_mouse_check = Instant::now() - Duration::from_secs(5);
                let mut last_mouse_attempt_path: Option<String> = None;
                let mut last_mouse_attempt_time: Option<Instant> = Some(Instant::now());
                let mut failed_paths: std::collections::HashMap<String, Instant> = std::collections::HashMap::new();
                while inner.running.load(Ordering::SeqCst) {
                    // Check and restart mouse hook if it has stopped
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
                            // Check whether the hook slot is occupied (crashed) or was never
                            // started (no device at launch). Either way, try to (re)start.
                            let hook_is_none = inner.mouse_hook.lock().unwrap().is_none();

                            // Only bother if a Logitech mouse evdev node is available now.
                            if let Some(mouse_path) = self::mouse_hook::find_logitech_mouse() {
                                // Implement throttled backoff to avoid busy loop logs if the device grab repeatedly fails (e.g. Device busy / permissions)
                                let should_attempt = match (&last_mouse_attempt_path, last_mouse_attempt_time) {
                                    (Some(last_path), Some(last_time)) if *last_path == mouse_path => {
                                        last_time.elapsed() >= Duration::from_secs(10)
                                    }
                                    _ => true,
                                };

                                if should_attempt {
                                    last_mouse_attempt_path = Some(mouse_path.clone());
                                    last_mouse_attempt_time = Some(Instant::now());

                                    log::info!("[Engine] Mouse hook not running but mouse detected — attempting to {}start...",
                                        if hook_is_none { "" } else { "re" });

                                    // Stop and drop any stale hook object.
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
                                            // Reset attempt cache on success
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
                    }

                    if last_reconnect.elapsed() >= Duration::from_secs(2) {
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

                        // Clean up failed_paths: remove paths that are no longer in `available`
                        failed_paths.retain(|path, _| {
                            available.iter().any(|info| info.path().to_string_lossy().to_string() == *path)
                        });

                        let mut clients = inner.hid_clients.lock().unwrap();

                        // Remove disconnected clients
                        clients.retain(|c| c.is_connected());

                        let mut new_keyboard_found = false;
                        for info in available {
                            let path_str = info.path().to_string_lossy().to_string();
                            let already_opened = clients.iter().any(|c| c.device_path == path_str);
                            if !already_opened {
                                // Apply cooldown (e.g. 10 seconds) on failed paths to avoid high-frequency retries/spams
                                if let Some(last_fail) = failed_paths.get(&path_str) {
                                    if last_fail.elapsed() < Duration::from_secs(10) {
                                        continue;
                                    }
                                }

                                let mut new_client = HidppClient::new();
                                if new_client.open_path(api, info.path()).is_ok() {
                                    log::info!("[Engine] HID++ device connected: {}", new_client.device_name);
                                    let layout = new_client.get_layout_key();
                                    if layout.starts_with("mx_keys") || layout.starts_with("mx_mechanical") {
                                        new_keyboard_found = true;
                                    }
                                    // Apply settings for mice
                                    if layout.starts_with("mx_master") || layout.starts_with("mx_anywhere") {
                                        let dpi = {
                                            let cfg = inner.config.lock().unwrap();
                                            cfg.settings.dpi as u32
                                        };
                                        let (ss_mode, ss_enabled, ss_threshold) = {
                                            let cfg = inner.config.lock().unwrap();
                                            (
                                                cfg.settings.smart_shift_mode.clone(),
                                                cfg.settings.smart_shift_enabled,
                                                cfg.settings.smart_shift_threshold as u8,
                                            )
                                        };
                                        let _ = new_client.set_dpi(dpi);
                                        let _ = new_client.set_smart_shift(&ss_mode, ss_enabled, ss_threshold);
                                    }
                                    clients.push(new_client);
                                    failed_paths.remove(&path_str);
                                } else {
                                    failed_paths.insert(path_str, Instant::now());
                                }
                            }
                        }

                        // Release clients lock before calling potentially slow or locking restart_keyboard_hooks
                        drop(clients);

                        if new_keyboard_found {
                            // Re-start keyboard hooks if a new keyboard was connected
                            let _ = engine_clone.restart_keyboard_hooks();
                        }
                    }

                    // Poll all clients
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

    pub fn stop(&self) {
        if !self.inner.running.load(Ordering::SeqCst) {
            return;
        }
        self.inner.running.store(false, Ordering::SeqCst);

        // Stop App Detector
        if let Some(mut app_det) = self.inner.app_detector.lock().unwrap().take() {
            app_det.stop();
        }

        // Stop Mouse Hook (releases exclusive evdev grab)
        if let Some(mut hook) = self.inner.mouse_hook.lock().unwrap().take() {
            hook.stop();
        }

        // Stop Keyboard Hooks
        let mut kb_hooks = self.inner.keyboard_hooks.lock().unwrap();
        for mut hook in kb_hooks.drain(..) {
            hook.stop();
        }

        // Stop HID Clients
        let mut clients = self.inner.hid_clients.lock().unwrap();
        for client in clients.iter_mut() {
            client.close();
        }
        clients.clear();
        log::info!("[Engine] Daemon stopped cleanly");
    }

    fn handle_app_change(&self, exe_name: String) {
        let (profile_name, mappings) = {
            let cfg = self.inner.config.lock().unwrap();
            let target = cfg.get_profile_for_app(&exe_name);
            let mappings = cfg.get_resolved_mappings(&target);
            (target, mappings)
        }; // cfg lock dropped here

        let mut last_exe = self.inner.last_detected_exe.lock().unwrap();
        let mut current_profile = self.inner.current_profile.lock().unwrap();
        
        // Skip only if BOTH profile AND exe unchanged
        if *current_profile == profile_name && *last_exe == exe_name {
            return;
        }
        
        *last_exe = exe_name.clone();
        
        if *current_profile != profile_name {
            log::info!("[Engine] App {} → profile '{}'", exe_name, profile_name);
            *current_profile = profile_name.clone();
            drop(current_profile); // release before config lock
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

    fn handle_mouse_hook_event(&self, event: MouseHookEvent) {
        match event {
            MouseHookEvent::Button { key, down } => {
                let btn_key = if key == Key::BTN_MIDDLE {
                    "middle"
                } else if key == Key::BTN_SIDE {
                    "xbutton1"
                } else if key == Key::BTN_EXTRA {
                    "xbutton2"
                } else {
                    return;
                };

                let gestures_enabled = {
                    let active = self.inner.active_mappings.lock().unwrap();
                    let enabled_key = if btn_key == "middle" { "middle_gesture_enabled" }
                                     else if btn_key == "xbutton1" { "xbutton1_gesture_enabled" }
                                     else { "xbutton2_gesture_enabled" };

                    // Explicit flag: "true" → enabled, "false" → check further, absent → enabled.
                    let explicit = active.get(enabled_key).map(|s| s.as_str());
                    match explicit {
                        Some("false") => {
                            // Even if flag says false, enable gesture mode if any direction is mapped.
                            let prefix = if btn_key == "middle" { "middle_gesture_" }
                                         else if btn_key == "xbutton1" { "xbutton1_gesture_" }
                                         else { "xbutton2_gesture_" };
                            ["left", "right", "up", "down"].iter().any(|dir| {
                                let k = format!("{}{}", prefix, dir);
                                active.get(&k).map(|v| v != "none").unwrap_or(false)
                            })
                        }
                        Some("true") => true,
                        _ => true, // absent key → enabled by default
                    }
                };

                if gestures_enabled {
                    if down {
                        self.handle_gesture_down();
                        self.inner.gesture_state.lock().unwrap().button = Some(btn_key.to_string());
                    } else {
                        // Check if this button was the one that started the gesture
                        let is_current = {
                            let state = self.inner.gesture_state.lock().unwrap();
                            state.button.as_ref() == Some(&btn_key.to_string())
                        };
                        if is_current {
                            self.handle_gesture_up();
                        }
                    }
                    return;
                }

                let mapping = {
                    let active = self.inner.active_mappings.lock().unwrap();
                    active.get(btn_key).cloned()
                };

                if let Some(action_id) = mapping {
                    if action_id == "none" {
                        return;
                    }
                    if is_mouse_button_action(&action_id) {
                        if let Some(sim_key) = get_mouse_button_key(&action_id) {
                            if down {
                                self.inner.key_simulator.inject_mouse_down(sim_key);
                            } else {
                                self.inner.key_simulator.inject_mouse_up(sim_key);
                            }
                        }
                    } else if down {
                        self.execute_engine_action(&action_id);
                    }
                }
            }
            MouseHookEvent::Scroll { horizontal, delta } => {
                if horizontal {
                    let action_key = if delta < 0 {
                        "hscroll_right"
                    } else {
                        "hscroll_left"
                    };

                    let mapping = {
                        let active = self.inner.active_mappings.lock().unwrap();
                        active.get(action_key).cloned()
                    };

                    if let Some(action_id) = mapping {
                        if action_id != "none" {
                            self.handle_hscroll_event(delta, &action_id);
                        }
                    }
                }
            }
            MouseHookEvent::Relative { dx, dy } => {
                if self.inner.gesture_active_arc.load(Ordering::SeqCst) {
                    self.handle_gesture_move(dx as i16, dy as i16, "evdev");
                }
            }
        }
    }

    fn handle_hid_event(&self, event: HidppEvent) {
        match event {
            HidppEvent::GestureDown => {
                let (enabled, mapping) = {
                    let active = self.inner.active_mappings.lock().unwrap();
                    let enabled = match active.get("gesture_enabled").map(|s| s.as_str()) {
                        Some("false") => {
                            // Auto-detect: enable if any direction is configured.
                            ["gesture_left", "gesture_right", "gesture_up", "gesture_down"]
                                .iter().any(|k| active.get(*k).map(|v| v != "none").unwrap_or(false))
                        }
                        _ => true,
                    };
                    let mapping = active.get("gesture").cloned();
                    (enabled, mapping)
                };
                if enabled {
                    self.handle_gesture_down();
                } else if let Some(ref action_id) = mapping {
                    if action_id != "none" {
                        if is_mouse_button_action(action_id) {
                            if let Some(sim_key) = get_mouse_button_key(action_id) {
                                log::info!("[Engine] Pressing mouse button for gesture action: {:?}", sim_key);
                                self.inner.key_simulator.inject_mouse_down(sim_key);
                            }
                        } else {
                            log::info!("[Engine] Executing gesture simple click action: {}", action_id);
                            self.execute_engine_action(action_id);
                        }
                    }
                }
            }
            HidppEvent::GestureUp => {
                let (enabled, mapping) = {
                    let active = self.inner.active_mappings.lock().unwrap();
                    let enabled = match active.get("gesture_enabled").map(|s| s.as_str()) {
                        Some("false") => {
                            ["gesture_left", "gesture_right", "gesture_up", "gesture_down"]
                                .iter().any(|k| active.get(*k).map(|v| v != "none").unwrap_or(false))
                        }
                        _ => true,
                    };
                    let mapping = active.get("gesture").cloned();
                    (enabled, mapping)
                };
                if enabled {
                    self.handle_gesture_up();
                } else if let Some(ref action_id) = mapping {
                    if action_id != "none" && is_mouse_button_action(action_id) {
                        if let Some(sim_key) = get_mouse_button_key(action_id) {
                            log::info!("[Engine] Releasing mouse button for gesture action: {:?}", sim_key);
                            self.inner.key_simulator.inject_mouse_up(sim_key);
                        }
                    }
                }
            }
            HidppEvent::GestureMove { dx, dy } => {
                let enabled = {
                    let btn_key = self.inner.gesture_state.lock().unwrap().button.clone();
                    let active = self.inner.active_mappings.lock().unwrap();
                    let enabled_key = match btn_key.as_deref() {
                        Some("middle")   => "middle_gesture_enabled",
                        Some("xbutton1") => "xbutton1_gesture_enabled",
                        Some("xbutton2") => "xbutton2_gesture_enabled",
                        _               => "gesture_enabled",
                    };
                    // "false" → check auto-detect (any direction configured)
                    match active.get(enabled_key).map(|s| s.as_str()) {
                        Some("false") => {
                            let prefix = match btn_key.as_deref() {
                                Some("middle")   => "middle_gesture_",
                                Some("xbutton1") => "xbutton1_gesture_",
                                Some("xbutton2") => "xbutton2_gesture_",
                                _               => "gesture_",
                            };
                            ["left", "right", "up", "down"].iter().any(|dir| {
                                let k = format!("{}{}", prefix, dir);
                                active.get(&k).map(|v| v != "none").unwrap_or(false)
                            })
                        }
                        _ => true,
                    }
                };
                if enabled {
                    self.handle_gesture_move(dx, dy, "hid_rawxy");
                } else {
                    self.inner.key_simulator.inject_relative_move(dx as i32, dy as i32);
                }
            }
            HidppEvent::ModeShiftDown => {
                log::debug!("[Engine] HID ModeShift button down");
                let mapping = self.inner.active_mappings.lock().unwrap()
                    .get("mode_shift").cloned();
                if let Some(ref action_id) = mapping {
                    self.execute_engine_action(action_id);
                }
            }
            HidppEvent::ModeShiftUp => {
                log::debug!("[Engine] HID ModeShift button up");
            }
        }
    }

    fn handle_gesture_down(&self) {
        log::debug!("[Engine] Gesture track: button down");
        self.inner.gesture_active_arc.store(true, Ordering::SeqCst);
        self.inner.gesture_tracking.store(true, Ordering::SeqCst);
        self.inner.gesture_triggered.store(false, Ordering::SeqCst);
        {
            let mut state = self.inner.gesture_state.lock().unwrap();
            state.delta_x = 0.0;
            state.delta_y = 0.0;
            state.last_move_at = Instant::now();
            state.input_source = None;
        }
    }

    fn handle_gesture_up(&self) {
        log::debug!("[Engine] Gesture track: button up");
        if !self.inner.gesture_active_arc.swap(false, Ordering::SeqCst) {
            return;
        }
        let triggered = self.inner.gesture_triggered.load(Ordering::SeqCst);
        self.inner.gesture_tracking.store(false, Ordering::SeqCst);

        if !triggered {
            let btn_key = {
                let state = self.inner.gesture_state.lock().unwrap();
                state.button.clone().unwrap_or_else(|| "gesture".to_string())
            };

            let mapping = self.inner.active_mappings.lock().unwrap()
                .get(&btn_key).cloned();
            if let Some(ref action_id) = mapping {
                log::info!("[Engine] Executing {} click fallback action: {}", btn_key, action_id);
                self.execute_engine_action(action_id);
            }
        }
        {
            let mut state = self.inner.gesture_state.lock().unwrap();
            state.input_source = None;
            state.button = None;
        }
    }

    fn handle_gesture_move(&self, dx: i16, dy: i16, source: &str) {
        let threshold = self.inner.cached_gesture_threshold.load(Ordering::Relaxed) as f32;
        let deadzone = self.inner.cached_gesture_deadzone.load(Ordering::Relaxed) as f32;
        let timeout = Duration::from_millis(self.inner.cached_gesture_timeout_ms.load(Ordering::Relaxed));
        let cooldown = Duration::from_millis(self.inner.cached_gesture_cooldown_ms.load(Ordering::Relaxed));

        let now = Instant::now();

        let mut state = self.inner.gesture_state.lock().unwrap();

        // 1. Cooldown check
        if now < state.cooldown_until {
            return;
        }

        // 2. Segment timeout check
        let idle_time = now.duration_since(state.last_move_at);
        if idle_time > timeout {
            log::debug!("[Engine] Segment timeout, resetting accumulator");
            self.inner.gesture_tracking.store(true, Ordering::SeqCst);
            self.inner.gesture_triggered.store(false, Ordering::SeqCst);
            state.delta_x = 0.0;
            state.delta_y = 0.0;
        }
        state.last_move_at = now;

        // 3. Source locking
        if let Some(ref active_source) = state.input_source {
            if active_source == "evdev" && source == "hid_rawxy" {
                log::debug!("[Engine] Promoting source to hid_rawxy");
                state.input_source = Some(source.to_string());
                state.delta_x = 0.0;
                state.delta_y = 0.0;
            } else if active_source != source {
                return;
            }
        } else {
            state.input_source = Some(source.to_string());
        }

        if !self.inner.gesture_tracking.load(Ordering::SeqCst) {
            self.inner.gesture_tracking.store(true, Ordering::SeqCst);
            self.inner.gesture_triggered.store(false, Ordering::SeqCst);
            state.delta_x = 0.0;
            state.delta_y = 0.0;
        }

        state.delta_x += dx as f32;
        state.delta_y += dy as f32;

        let accum_x = state.delta_x;
        let accum_y = state.delta_y;

        let abs_x = accum_x.abs();
        let abs_y = accum_y.abs();
        let dominant = abs_x.max(abs_y);

        if dominant < threshold {
            return;
        }

        if self.inner.gesture_triggered.load(Ordering::SeqCst) {
            return;
        }

        let cross_limit = deadzone.max(dominant * 0.35);

        let gesture_action = if abs_x > abs_y {
            if abs_y > cross_limit {
                None
            } else if accum_x > 0.0 {
                Some("gesture_right")
            } else {
                Some("gesture_left")
            }
        } else {
            if abs_x > cross_limit {
                None
            } else if accum_y > 0.0 {
                Some("gesture_down")
            } else {
                Some("gesture_up")
            }
        };

        if let Some(action_key) = gesture_action {
            self.inner.gesture_triggered.store(true, Ordering::SeqCst);
            state.cooldown_until = now + cooldown;

            let btn_key = state.button.clone().unwrap_or_else(|| "gesture".to_string());

            // Drop state lock before calling potentially blocking or locking operations (execute_engine_action, active_mappings, etc.)
            drop(state);

            let resolved_action_key = if btn_key == "gesture" {
                action_key.to_string()
            } else {
                format!("{}_{}", btn_key, action_key)
            };

            log::info!("[Engine] Gesture detected for {}: {}", btn_key, resolved_action_key);

            let mapping = self.inner.active_mappings.lock().unwrap()
                .get(&resolved_action_key).cloned();
            if let Some(ref action_id) = mapping {
                self.execute_engine_action(action_id);
            }
        }
    }

    fn handle_hscroll_event(&self, delta: i32, action_id: &str) {
        let threshold = {
            let cfg = self.inner.config.lock().unwrap();
            cfg.settings.hscroll_threshold as f32
        };
        let now = Instant::now();

        let is_volume = action_id == "volume_up" || action_id == "volume_down";
        let cooldown = if is_volume {
            Duration::from_millis(60)
        } else {
            Duration::from_millis(350)
        };

        let (accum_ref, last_fire_ref) = if delta < 0 {
            (&self.inner.hscroll_accum_right, &self.inner.hscroll_last_fire_right)
        } else {
            (&self.inner.hscroll_accum_left, &self.inner.hscroll_last_fire_left)
        };

        {
            let last_fire = last_fire_ref.lock().unwrap();
            if now.duration_since(*last_fire) < cooldown {
                let mut accum = accum_ref.lock().unwrap();
                *accum = 0.0;
                return;
            }
        }

        let step = (delta.abs() as f32).min(1.0);
        let mut accum = accum_ref.lock().unwrap();
        *accum += step;

        if *accum < threshold {
            return;
        }

        *accum = 0.0;
        *last_fire_ref.lock().unwrap() = now;

        log::info!("[Engine] HScroll action triggered: {}", action_id);
        self.execute_engine_action(action_id);
    }

    fn execute_engine_action(&self, action_id: &str) {
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

    fn toggle_smart_shift(&self) {
        let (mode, enabled, threshold) = {
            let mut cfg = self.inner.config.lock().unwrap();
            let next_enabled = !cfg.settings.smart_shift_enabled;
            cfg.settings.smart_shift_enabled = next_enabled;
            let _ = cfg.save();
            self.increment_config_generation();
            (cfg.settings.smart_shift_mode.clone(), next_enabled, cfg.settings.smart_shift_threshold as u8)
        };
        log::info!("[Engine] Toggling SmartShift: enabled={}", enabled);

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock().unwrap();
            for client in clients.iter_mut() {
                if client.is_connected() && (client.get_layout_key().starts_with("mx_master")) {
                    if let Err(e) = client.set_smart_shift(&mode, enabled, threshold) {
                        log::error!("[Engine] Failed to set SmartShift on {}: {}", client.device_name, e);
                    }
                }
            }
        });
    }

    fn switch_scroll_mode(&self) {
        let (mode, threshold) = {
            let mut cfg = self.inner.config.lock().unwrap();
            let next_mode = if cfg.settings.smart_shift_mode == "ratchet" { "freespin" } else { "ratchet" };
            cfg.settings.smart_shift_mode = next_mode.to_string();
            cfg.settings.smart_shift_enabled = false;
            let _ = cfg.save();
            self.increment_config_generation();
            (next_mode.to_string(), cfg.settings.smart_shift_threshold as u8)
        };
        log::info!("[Engine] Switching scroll mode to ratchet/freespin fixed: mode={}", mode);

        let inner_clone = self.inner.clone();
        thread::spawn(move || {
            let mut clients = inner_clone.hid_clients.lock().unwrap();
            for client in clients.iter_mut() {
                if client.is_connected() && (client.get_layout_key().starts_with("mx_master")) {
                    if let Err(e) = client.set_smart_shift(&mode, false, threshold) {
                        log::error!("[Engine] Failed to set scroll mode on {}: {}", client.device_name, e);
                    }
                }
            }
        });
    }

    fn cycle_dpi(&self) {
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
                if client.is_connected() && (client.get_layout_key().starts_with("mx_master") || client.get_layout_key().starts_with("mx_anywhere")) {
                    if let Err(e) = client.set_dpi(new_dpi as u32) {
                        log::error!("[Engine] Failed to write DPI to {}: {}", client.device_name, e);
                    }
                }
            }
        });
    }

    pub fn get_config(&self) -> Config {
        self.inner.config.lock().unwrap().clone()
    }

    pub fn active_profile_shared(&self) -> Arc<Mutex<String>> {
        self.inner.active_profile_shared.clone()
    }

    pub fn config_generation(&self) -> u64 {
        self.inner.config_generation.load(Ordering::Relaxed)
    }

    pub fn increment_config_generation(&self) {
        self.inner.config_generation.fetch_add(1, Ordering::Relaxed);
    }

    pub fn device_connected(&self) -> bool {
        !self.inner.hid_clients.lock().unwrap().is_empty()
    }

    pub fn device_names(&self) -> Vec<String> {
        let clients = self.inner.hid_clients.lock().unwrap();
        clients.iter().map(|c| c.device_name.clone()).collect()
    }

    pub fn selected_device_name(&self) -> String {
        let clients = self.inner.hid_clients.lock().unwrap();
        let idx = *self.inner.selected_device_idx.lock().unwrap();
        if let Some(c) = clients.get(idx) {
            c.device_name.clone()
        } else {
            "None".to_string()
        }
    }

    pub fn selected_device_layout(&self) -> String {
        let clients = self.inner.hid_clients.lock().unwrap();
        let idx = *self.inner.selected_device_idx.lock().unwrap();
        if let Some(c) = clients.get(idx) {
            let layout = c.get_layout_key();
            log::debug!("[Engine] Selected device layout: idx={}, name={}, layout={}", idx, c.device_name, layout);
            layout
        } else {
            log::debug!("[Engine] Selected device layout: idx={}, no client (returning generic)", idx);
            "generic".to_string()
        }
    }

    pub fn set_selected_device(&self, idx: usize) {
        *self.inner.selected_device_idx.lock().unwrap() = idx;
    }

    pub fn select_profile(&self, name: &str) {
        log::info!("[Engine] Selecting profile: {}", name);
        {
            let mut cfg = self.inner.config.lock().unwrap();
            cfg.active_profile = name.to_string();
            let _ = cfg.save();
            self.increment_config_generation();
        }
        self.refresh_active_profile();
    }

    pub fn add_profile(&self, name: &str) {
        log::info!("[Engine] Adding profile: {}", name);
        {
            let mut cfg = self.inner.config.lock().unwrap();
            if !cfg.profiles.contains_key(name) {
                let default_profile = cfg.profiles.get("default").cloned().unwrap_or_else(|| {
                    self::config::Profile {
                        label: name.to_string(),
                        apps: Vec::new(),
                        mappings: std::collections::HashMap::new(),
                    }
                });
                let mut new_profile = default_profile;
                new_profile.label = name.to_string();
                new_profile.apps = Vec::new();
                cfg.profiles.insert(name.to_string(), new_profile);
                let _ = cfg.save();
                self.increment_config_generation();
            }
        }
    }

    pub fn delete_profile(&self, name: &str) {
        log::info!("[Engine] Deleting profile: {}", name);
        if name == "default" {
            return;
        }
        {
            let mut cfg = self.inner.config.lock().unwrap();
            cfg.profiles.remove(name);
            if cfg.active_profile == name {
                cfg.active_profile = "default".to_string();
            }
            let _ = cfg.save();
            self.increment_config_generation();
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
            if let Some(profile) = cfg.profiles.get_mut(profile_name) {
                profile.apps = apps;
                let _ = cfg.save();
                self.increment_config_generation();
            }
        }
    }

    pub fn update_profile_mappings(&self, profile_name: &str, mappings: std::collections::HashMap<String, String>) {
        log::info!("[Engine] Updating mappings for profile {}", profile_name);
        {
            let mut cfg = self.inner.config.lock().unwrap();
            if let Some(profile) = cfg.profiles.get_mut(profile_name) {
                for (k, v) in mappings {
                    profile.mappings.insert(k, v);
                }
                let _ = cfg.save();
                self.increment_config_generation();
            }
        }
        log::info!("[Engine] Saved mappings, refreshing active profile...");
        self.refresh_active_profile();
        log::info!("[Engine] Refreshed active profile, restarting keyboard hooks...");
        let _ = self.restart_keyboard_hooks();
        log::info!("[Engine] Keyboard hooks restarted successfully!");
    }

    fn restart_keyboard_hooks(&self) -> anyhow::Result<()> {
        // Drop config lock before acquiring keyboard_hooks lock
        let mappings = Arc::new(std::sync::Mutex::new(
            self.inner.config.lock().unwrap().get_active_mappings()
        ));
        let mut kb_hooks = self.inner.keyboard_hooks.lock().unwrap();
        for hook in kb_hooks.iter_mut() { hook.stop(); }
        kb_hooks.clear();
        let kb_paths = crate::keyboard_hook::find_logitech_keyboards();
        for path in kb_paths {
            let mut hook = KeyboardHook::new();
            if hook.start(path, mappings.clone(), self.inner.key_simulator.device(), self.inner.key_simulator.clone()).is_ok() {
                kb_hooks.push(hook);
            }
        }
        Ok(())
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
    ) {
        log::info!(
            "[Engine] Updating global settings: DPI={}, SmartShift mode={}, enabled={}, threshold={}, invert_hscroll={}, invert_vscroll={}, gesture_threshold={}, gesture_deadzone={}, accent_color={}",
            dpi, smart_shift_mode, smart_shift_enabled, smart_shift_threshold, invert_hscroll, invert_vscroll, gesture_threshold, gesture_deadzone, accent_color
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
            let _ = cfg.save();
            self.increment_config_generation();
        }

        self.inner.invert_vscroll_arc.store(invert_vscroll, Ordering::SeqCst);
        self.inner.invert_hscroll_arc.store(invert_hscroll, Ordering::SeqCst);
        // Keep lock-free gesture settings cache in sync
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
