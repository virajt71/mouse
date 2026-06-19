pub mod clipboard;
pub mod network;
pub mod switching;

use crate::lock_ext::MutexExt;
use crate::config::Config;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

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
        let inner_switch = engine_inner.clone();
        std::thread::spawn(move || {
            switching::run_edge_detection_loop(inner_switch);
        });
    }

    pub fn is_forwarding_to_remote(&self) -> bool {
        self.is_forwarding.load(Ordering::SeqCst)
    }

    pub fn get_active_peer_name(&self) -> Option<String> {
        self.active_peer.read().unwrap().clone()
    }

    pub fn set_active_peer(&self, peer: Option<String>) {
        let was_local = self.active_peer.read().unwrap().is_none();
        let going_remote = peer.is_some();
        self.is_forwarding.store(going_remote, Ordering::SeqCst);

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
            .store(cfg.settings.flow_enabled, Ordering::SeqCst);
        *self.flow_mouse_mode.write().unwrap() = cfg.settings.flow_mouse_mode.clone();
        self.flow_mouse_mode_hardware
            .store(cfg.settings.flow_mouse_mode == "hardware", Ordering::SeqCst);
        *self.flow_hold_key.write().unwrap() = cfg.settings.flow_hold_key.clone();
        *self.flow_peers.write().unwrap() = cfg.settings.flow_peers.clone();
    }

    pub fn handle_raw_motion(&self, dx: i32, dy: i32) -> Option<String> {
        if !self.flow_mouse_mode_hardware.load(Ordering::SeqCst) {
            return None;
        }

        let mut vx = self.virtual_x.lock_safe();
        let mut vy = self.virtual_y.lock_safe();

        let sw = *self.screen_width.read().unwrap();
        let sh = *self.screen_height.read().unwrap();

        *vx = (*vx + dx).clamp(0, sw);
        *vy = (*vy + dy).clamp(0, sh);

        let threshold = 5;
        let mut transition_to = None;

        // Boundary checks
        if *vx <= threshold {
            transition_to = Some((-1, 0)); // Left
        } else if *vx >= sw - threshold {
            transition_to = Some((1, 0)); // Right
        } else if *vy <= threshold {
            transition_to = Some((0, -1)); // Top
        } else if *vy >= sh - threshold {
            transition_to = Some((0, 1)); // Bottom
        }

        if let Some((lx, ly)) = transition_to {
            if self.flow_enabled.load(Ordering::SeqCst) {
                let peers = self.flow_peers.read().unwrap();
                for peer in peers.iter() {
                    if peer.paired && peer.layout_x == lx && peer.layout_y == ly {
                        // Reset virtual coordinates to opposite edge to prevent loop bouncing
                        if lx == -1 {
                            *vx = sw - 20;
                        } else if lx == 1 {
                            *vx = 20;
                        } else if ly == -1 {
                            *vy = sh - 20;
                        } else if ly == 1 {
                            *vy = 20;
                        }
                        return Some(peer.name.clone());
                    }
                }
            }
        }

        None
    }
}
