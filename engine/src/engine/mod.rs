pub mod action;
pub mod app_change;
pub mod gesture;
pub mod hotplug;
pub mod hscroll;
pub mod inner;
pub mod modifier_state;
pub mod profile;

use crate::lock_ext::MutexExt;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::Instant;

use crate::config::Config;
use crate::detection::AppDetector;
use crate::input::KeySimulator;

pub use self::inner::{EngineInner, GestureState};
pub use self::modifier_state::ModifierState;

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
        if let Some(layout) = config
            .settings
            .device_layout_overrides
            .get("keyboard_layout")
            .and_then(|v| v.as_str())
        {
            key_simulator.set_keyboard_layout(layout);
        }
        let current_profile = "global".to_string();

        let invert_vscroll = config.settings.invert_vscroll;
        let invert_hscroll = config.settings.invert_hscroll;
        let init_gesture_threshold = config.settings.gesture_threshold.max(0) as u32;
        let init_gesture_deadzone = config.settings.gesture_deadzone.max(0) as u32;
        let init_gesture_timeout_ms = config.settings.gesture_timeout_ms;
        let init_gesture_cooldown_ms = config.settings.gesture_cooldown_ms;
        let init_hscroll_threshold = config.settings.hscroll_threshold.max(0) as u32;

        let inner = EngineInner {
            cached_device_state: Mutex::new(inner::CachedDeviceState::default()),
            config: Mutex::new(config),
            config_generation: AtomicU64::new(1),
            key_simulator,
            app_detector: Mutex::new(None),
            mouse_hooks: Mutex::new(Vec::new()),
            keyboard_hooks: Mutex::new(Vec::new()),
            hid_api: Mutex::new(hidapi::HidApi::new().ok()),
            hid_clients: Mutex::new(Vec::new()),
            selected_device_idx: Mutex::new(0),
            running: AtomicBool::new(false),
            current_profile: Mutex::new(current_profile.clone()),
            active_mappings: Arc::new(std::sync::RwLock::new(HashMap::new())),
            active_profile_shared: Arc::new(Mutex::new(current_profile)),
            last_detected_exe: Mutex::new(String::new()),

            blocked_buttons_arc: Arc::new(Mutex::new(Vec::new())),
            invert_vscroll_arc: Arc::new(AtomicBool::new(invert_vscroll)),
            invert_hscroll_arc: Arc::new(AtomicBool::new(invert_hscroll)),
            block_hscroll_arc: Arc::new(AtomicBool::new(false)),
            gesture_active_arc: Arc::new(AtomicBool::new(false)),
            modifier_state: ModifierState::new(),

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
            cached_hscroll_threshold: AtomicU32::new(init_hscroll_threshold),
            config_change_listener: Mutex::new(None),
        };

        let engine = Engine {
            inner: Arc::new(inner),
        };
        engine.refresh_active_profile();
        engine
    }

    pub fn start(&self) -> anyhow::Result<()> {
        if self.inner.running.load(Ordering::Acquire) {
            return Ok(());
        }
        self.inner.running.store(true, Ordering::Release);

        self.inner.key_simulator.ensure_device();

        self.inner.gesture_active_arc.store(false, Ordering::Relaxed);
        self.inner.gesture_tracking.store(false, Ordering::Relaxed);
        self.inner.gesture_triggered.store(false, Ordering::Relaxed);

        self.refresh_active_profile();

        let _ = self.restart_keyboard_hooks();



        let mut app_det_lock = self.inner.app_detector.lock_safe();
        if app_det_lock.is_none() {
            let engine = self.clone();
            let mut app_det = AppDetector::new(move |exe_name| {
                engine.handle_app_change(exe_name);
            });
            app_det.start();
            *app_det_lock = Some(app_det);
        }

        crate::flow::FLOW_MANAGER.start(self.inner.clone());

        self.spawn_hidpp_thread()?;
        Ok(())
    }

    pub fn stop(&self) {
        if !self.inner.running.load(Ordering::Acquire) {
            return;
        }
        self.inner.running.store(false, Ordering::Release);

        if let Some(mut app_det) = self.inner.app_detector.lock_safe().take() {
            app_det.stop();
        }

        let mut mouse_hooks = self.inner.mouse_hooks.lock_safe();
        for mut hook in mouse_hooks.drain(..) {
            hook.stop();
        }

        let mut kb_hooks = self.inner.keyboard_hooks.lock_safe();
        for mut hook in kb_hooks.drain(..) {
            hook.stop();
        }

        let mut clients = self.inner.hid_clients.lock_safe();
        for client in clients.iter_mut() {
            client.close();
        }
        clients.clear();
        drop(clients); // Unlock before calling update to avoid any re-locking overhead
        self.update_cached_device_state();
        log::info!("[Engine] Daemon stopped cleanly");
    }

    pub fn get_config(&self) -> Config {
        self.inner.config.lock_safe().clone()
    }

    pub fn get_config_if_changed(&self, last_gen: u64) -> Option<Config> {
        let current_gen = self.config_generation();
        if current_gen > last_gen {
            Some(self.get_config())
        } else {
            None
        }
    }

    pub fn active_profile_shared(&self) -> Arc<Mutex<String>> {
        self.inner.active_profile_shared.clone()
    }

    pub fn config_generation(&self) -> u64 {
        self.inner.config_generation.load(Ordering::Relaxed)
    }

    pub fn increment_config_generation(&self, cfg: &Config) {
        crate::flow::FLOW_MANAGER.update_config(cfg);
        let gen = self.inner.config_generation.fetch_add(1, Ordering::Relaxed) + 1;
        if let Ok(lock) = self.inner.config_change_listener.lock() {
            if let Some(ref callback) = *lock {
                callback(cfg, gen);
            }
        }
    }

    pub fn set_config_change_listener<F>(&self, listener: F)
    where
        F: Fn(&Config, u64) + Send + Sync + 'static,
    {
        *self.inner.config_change_listener.lock_safe() = Some(Box::new(listener));
    }

    pub fn update_cached_device_state(&self) {
        let clients = self.inner.hid_clients.lock_safe();
        let idx = *self.inner.selected_device_idx.lock_safe();

        let device_connected = !clients.is_empty();
        let device_names: Vec<String> = clients.iter().map(|c| c.device_name.clone()).collect();

        let (selected_device_name, selected_device_layout, active_host_channel) = if let Some(c) = clients.get(idx) {
            let name = c.device_name.clone();
            let layout = c.get_layout_key();
            let channel = c.active_host_channel();
            (name, layout, channel)
        } else {
            ("None".to_string(), "generic".to_string(), None)
        };

        let mut cached = self.inner.cached_device_state.lock_safe();
        cached.device_connected = device_connected;
        cached.device_names = device_names;
        cached.selected_device_name = selected_device_name;
        cached.selected_device_layout = selected_device_layout;
        cached.active_host_channel = active_host_channel;
    }

    pub fn device_connected(&self) -> bool {
        self.inner.cached_device_state.lock_safe().device_connected
    }

    pub fn device_names(&self) -> Vec<String> {
        self.inner.cached_device_state.lock_safe().device_names.clone()
    }

    pub fn selected_device_name(&self) -> String {
        self.inner.cached_device_state.lock_safe().selected_device_name.clone()
    }

    pub fn active_host_channel(&self) -> Option<u8> {
        self.inner.cached_device_state.lock_safe().active_host_channel
    }

    pub fn selected_device_layout(&self) -> String {
        self.inner.cached_device_state.lock_safe().selected_device_layout.clone()
    }

    pub fn set_selected_device(&self, idx: usize) {
        *self.inner.selected_device_idx.lock_safe() = idx;
        self.update_cached_device_state();
    }
}
