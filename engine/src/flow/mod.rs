pub mod clipboard;
pub mod network;
pub mod switching;
pub mod edge;
pub mod topology;
pub mod handoff;
pub mod channel_switch;

use crate::lock_ext::MutexExt;
use crate::config::Config;
use std::sync::atomic::{AtomicBool, Ordering, AtomicU8, AtomicI32, AtomicU64};
use std::sync::{Arc, Mutex, RwLock};
use edge::EdgeEvent;

#[allow(non_upper_case_globals)]
pub static FLOW_MANAGER: std::sync::LazyLock<Arc<FlowManager>> =
    std::sync::LazyLock::new(|| Arc::new(FlowManager::new()));

pub struct FlowManager {
    pub active_peer: RwLock<Option<String>>, // None means local input is active
    pub virtual_x: Mutex<i32>,
    pub virtual_y: Mutex<i32>,
    pub screen_width: RwLock<i32>,
    pub screen_height: RwLock<i32>,
    pub is_running: Mutex<bool>,
    pub engine_inner: Mutex<Option<Arc<crate::engine::inner::EngineInner>>>,
    pub current_controller: RwLock<Option<String>>,

    // Cached configuration fields to avoid locking config on every mouse event
    pub flow_enabled: AtomicBool,
    pub flow_mouse_mode: RwLock<String>,
    pub flow_hold_key: RwLock<String>,
    pub flow_peers: RwLock<Vec<crate::config::FlowPeer>>,
    pub is_forwarding: AtomicBool,
    pub flow_mouse_mode_hardware: AtomicBool,
    pub flow_local_name: RwLock<String>,
    pub flow_local_channel_index: AtomicU8,
    pub flow_edge_threshold: AtomicI32,
    pub flow_hold_ctrl_only: AtomicBool,
    pub flow_handoff_timeout_ms: AtomicU64,
}

impl FlowManager {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        FlowManager {
            active_peer: RwLock::new(None),
            virtual_x: Mutex::new(960), // start in center of screen
            virtual_y: Mutex::new(540),
            screen_width: RwLock::new(1920),
            screen_height: RwLock::new(1080),
            is_running: Mutex::new(false),
            engine_inner: Mutex::new(None),
            current_controller: RwLock::new(None),
            flow_enabled: AtomicBool::new(false),
            flow_mouse_mode: RwLock::new("software".to_string()),
            flow_hold_key: RwLock::new("none".to_string()),
            flow_peers: RwLock::new(Vec::new()),
            is_forwarding: AtomicBool::new(false),
            flow_mouse_mode_hardware: AtomicBool::new(false),
            flow_local_name: RwLock::new("".to_string()),
            flow_local_channel_index: AtomicU8::new(0),
            flow_edge_threshold: AtomicI32::new(5),
            flow_hold_ctrl_only: AtomicBool::new(false),
            flow_handoff_timeout_ms: AtomicU64::new(500),
        }
    }

    pub fn start(&self, engine_inner: Arc<crate::engine::inner::EngineInner>) {
        let mut running = self.is_running.lock_safe();
        if *running {
            return;
        }
        *running = true;

        log::info!("[Flow] Starting FlowManager background threads...");

        // Update cached config
        {
            let cfg = engine_inner.config.lock_safe();
            self.update_config(&cfg);
        }

        // Store engine_inner
        *self.engine_inner.lock_safe() = Some(engine_inner.clone());

        // 1. Start UDP Discovery thread
        let inner_disc = engine_inner.clone();
        std::thread::spawn(move || {
            network::run_discovery_loop(inner_disc);
        });

        // 2. Start TCP TLS Server thread
        let inner_serv = engine_inner.clone();
        std::thread::spawn(move || {
            network::run_server_loop(inner_serv);
        });

        // 3. Start Clipboard Sync thread
        std::thread::spawn(move || {
            clipboard::run_clipboard_loop();
        });

        // 4. Start Edge Detection thread
        let inner_edge = engine_inner.clone();
        std::thread::spawn(move || {
            edge::run_x11_edge_polling(inner_edge);
        });

        // 5. Start Handoff channel listener thread
        std::thread::spawn(move || {
            handoff::run_handoff_loop();
        });
    }

    pub fn is_forwarding_to_remote(&self) -> bool {
        self.is_forwarding.load(Ordering::Relaxed)
    }

    pub fn get_active_peer_name(&self) -> Option<String> {
        self.active_peer.read().unwrap().clone()
    }

    pub fn set_active_peer(&self, peer: Option<String>) {
        let was_local = self.active_peer.read().unwrap().is_none();
        let going_remote = peer.is_some();
        self.is_forwarding.store(going_remote, Ordering::Relaxed);

        if was_local && going_remote {
            // Flush modifiers before handing off
            if let Some(ref inner) = *self.engine_inner.lock_safe() {
                use evdev::Key;
                let mods = [
                    Key::KEY_LEFTCTRL,
                    Key::KEY_RIGHTCTRL,
                    Key::KEY_LEFTSHIFT,
                    Key::KEY_RIGHTSHIFT,
                    Key::KEY_LEFTALT,
                    Key::KEY_RIGHTALT,
                    Key::KEY_LEFTMETA,
                    Key::KEY_RIGHTMETA,
                ];
                for key in mods {
                    inner.key_simulator.inject_key_up(key);
                }
            }
        }

        let mut active = self.active_peer.write().unwrap();
        *active = peer;
    }

    pub fn get_current_controller(&self) -> Option<String> {
        self.current_controller.read().unwrap().clone()
    }

    pub fn set_current_controller(&self, peer: Option<String>) {
        *self.current_controller.write().unwrap() = peer;
    }

    pub fn update_config(&self, cfg: &Config) {
        *self.screen_width.write().unwrap() = cfg.settings.flow_screen_width;
        *self.screen_height.write().unwrap() = cfg.settings.flow_screen_height;
        self.flow_enabled
            .store(cfg.settings.flow_enabled, Ordering::Relaxed);
        *self.flow_mouse_mode.write().unwrap() = cfg.settings.flow_mouse_mode.clone();
        self.flow_mouse_mode_hardware
            .store(cfg.settings.flow_mouse_mode == "hardware", Ordering::Relaxed);
        *self.flow_hold_key.write().unwrap() = cfg.settings.flow_hold_key.clone();
        *self.flow_peers.write().unwrap() = cfg.settings.flow_peers.clone();
        *self.flow_local_name.write().unwrap() = cfg.settings.flow_local_name.clone();
        self.flow_local_channel_index
            .store(cfg.settings.flow_local_channel_index, Ordering::Relaxed);
        self.flow_edge_threshold
            .store(cfg.settings.flow_edge_threshold, Ordering::Relaxed);
        self.flow_hold_ctrl_only
            .store(cfg.settings.flow_hold_ctrl_only, Ordering::Relaxed);
        self.flow_handoff_timeout_ms
            .store(cfg.settings.flow_handoff_timeout_ms, Ordering::Relaxed);
    }

    pub fn handle_raw_motion(&self, dx: i32, dy: i32) -> Option<EdgeEvent> {
        let mut vx = self.virtual_x.lock_safe();
        let mut vy = self.virtual_y.lock_safe();

        let sw = *self.screen_width.read().unwrap();
        let sh = *self.screen_height.read().unwrap();

        *vx = (*vx + dx).clamp(0, sw);
        *vy = (*vy + dy).clamp(0, sh);

        let threshold = self.flow_edge_threshold.load(Ordering::Relaxed);
        let mut edge = None;

        // Boundary checks
        if *vx <= threshold {
            edge = Some(EdgeEvent::Left);
        } else if *vx >= sw - threshold {
            edge = Some(EdgeEvent::Right);
        } else if *vy <= threshold {
            edge = Some(EdgeEvent::Top);
        } else if *vy >= sh - threshold {
            edge = Some(EdgeEvent::Bottom);
        }

        edge
    }

    pub fn get_active_hardware_peer_name(&self) -> Option<String> {
        let engine_opt = self.engine_inner.lock_safe();
        let engine_inner = engine_opt.as_ref()?;
        
        let current_active_channel = engine_inner.cached_device_state.lock_safe().active_host_channel;

        let ch = current_active_channel?;

        let peers = self.flow_peers.read().unwrap();
        peers.iter()
            .find(|p| p.paired && p.channel_index == ch)
            .map(|p| p.name.clone())
    }
}
