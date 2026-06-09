pub mod inner;
pub mod profile;
pub mod gesture;
pub mod hscroll;
pub mod action;
pub mod hotplug;
pub mod app_change;

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering, AtomicU32, AtomicU64}};
use std::collections::HashMap;
use std::time::Instant;

use crate::config::Config;
use crate::input::{KeySimulator, MouseHook};
use crate::detection::AppDetector;

pub use self::inner::{EngineInner, GestureState};

#[derive(Clone)]
pub struct Engine {
    pub inner: Arc<EngineInner>,
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
            hscroll_last_fire_left: Mutex::new(Instant::now() - std::time::Duration::from_secs(1)),
            hscroll_last_fire_right: Mutex::new(Instant::now() - std::time::Duration::from_secs(1)),

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

    pub fn start(&self) -> anyhow::Result<()> {
        if self.inner.running.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.inner.running.store(true, Ordering::SeqCst);

        self.inner.key_simulator.ensure_device();

        self.inner.gesture_active_arc.store(false, Ordering::SeqCst);
        self.inner.gesture_tracking.store(false, Ordering::SeqCst);
        self.inner.gesture_triggered.store(false, Ordering::SeqCst);

        self.refresh_active_profile();

        let _ = self.restart_keyboard_hooks();

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
                    }
                }
            }
        }

        let mut app_det_lock = self.inner.app_detector.lock().unwrap();
        if app_det_lock.is_none() {
            let engine = self.clone();
            let mut app_det = AppDetector::new(move |exe_name| {
                engine.handle_app_change(exe_name);
            });
            app_det.start();
            *app_det_lock = Some(app_det);
        }

        self.spawn_hidpp_thread()?;
        Ok(())
    }

    pub fn stop(&self) {
        if !self.inner.running.load(Ordering::SeqCst) {
            return;
        }
        self.inner.running.store(false, Ordering::SeqCst);

        if let Some(mut app_det) = self.inner.app_detector.lock().unwrap().take() {
            app_det.stop();
        }

        if let Some(mut hook) = self.inner.mouse_hook.lock().unwrap().take() {
            hook.stop();
        }

        let mut kb_hooks = self.inner.keyboard_hooks.lock().unwrap();
        for mut hook in kb_hooks.drain(..) {
            hook.stop();
        }

        let mut clients = self.inner.hid_clients.lock().unwrap();
        for client in clients.iter_mut() {
            client.close();
        }
        clients.clear();
        log::info!("[Engine] Daemon stopped cleanly");
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
}
