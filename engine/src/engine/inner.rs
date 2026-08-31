use evdev::Key;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::config::{Config, DeviceKey};
use crate::detection::AppDetector;
use crate::hidpp::HidppClient;
use crate::input::{KeySimulator, KeyboardHook, MouseHook};
use crate::lock_ext::MutexExt;

use super::modifier_state::ModifierState;

pub struct GestureState {
    pub delta_x: f32,
    pub delta_y: f32,
    pub last_move_at: Instant,
    pub cooldown_until: Instant,
    pub input_source: Option<String>,
    pub button: Option<String>,
}

#[derive(Clone, Default, Debug)]
pub struct CachedDeviceState {
    pub device_connected: bool,
    pub device_names: Vec<String>,
    pub selected_device_name: String,
    pub selected_device_layout: String,
    pub active_host_channel: Option<u8>,
}

/// Per-device resolved runtime state. Each physical device (keyed by
/// `DeviceKey`) keeps its own active profile + mappings so a per-app profile on
/// the mouse never bleeds onto the keyboard (and vice versa).
#[derive(Clone, Default, Debug)]
pub struct PerDeviceState {
    pub current_profile: String,
    pub active_profile_shared: String,
    pub last_detected_exe: String,
    pub active_mappings: Arc<std::sync::RwLock<HashMap<String, std::sync::Arc<str>>>>,
    pub blocked_buttons: Vec<Key>,
    pub block_hscroll: bool,
}

pub struct EngineInner {
    pub cached_device_state: Mutex<CachedDeviceState>,
    pub config: Mutex<Config>,
    pub config_generation: AtomicU64,
    pub key_simulator: KeySimulator,
    pub app_detector: Mutex<Option<AppDetector>>,
    pub mouse_hooks: Mutex<Vec<MouseHook>>,
    pub keyboard_hooks: Mutex<Vec<KeyboardHook>>,
    pub hid_api: Mutex<Option<hidapi::HidApi>>,
    pub hid_clients: Mutex<Vec<HidppClient>>,
    pub selected_device_idx: Mutex<usize>,
    pub running: AtomicBool,
    /// Per-device resolved state, keyed by `DeviceKey` (layout + HID++ name).
    pub per_device: Mutex<HashMap<DeviceKey, PerDeviceState>>,
    /// Shared fallback exe cache (kept for single-device callers / migration).
    pub last_detected_exe: Mutex<String>,

    // Shared state variables with MouseHook (global gesture settings)
    pub blocked_buttons_arc: Arc<Mutex<Vec<Key>>>,
    pub invert_vscroll_arc: Arc<AtomicBool>,
    pub invert_hscroll_arc: Arc<AtomicBool>,
    /// True when hscroll_left/right mapping is "none" — raw REL_HWHEEL passes through to the compositor.
    /// When false (an action is mapped), the engine handles hscroll via handle_hscroll_event and
    /// raw hscroll is suppressed so the action does not double-fire (§3.1).
    pub block_hscroll_arc: Arc<AtomicBool>,
    pub gesture_active_arc: Arc<AtomicBool>,

    // Gesture tracking state
    pub gesture_tracking: AtomicBool,
    pub gesture_triggered: AtomicBool,
    pub gesture_state: Mutex<GestureState>,

    // HScroll tracking state
    pub hscroll_accum_left: Mutex<f32>,
    pub hscroll_accum_right: Mutex<f32>,
    pub hscroll_last_fire_left: Mutex<Instant>,
    pub hscroll_last_fire_right: Mutex<Instant>,

    // Cached gesture settings
    pub cached_gesture_threshold: AtomicU32,
    pub cached_gesture_deadzone: AtomicU32,
    pub cached_gesture_timeout_ms: AtomicU64,
    pub cached_gesture_cooldown_ms: AtomicU64,

    // Cached scroll settings
    pub cached_hscroll_threshold: AtomicU32,

    /// Modifier key state updated by KeyboardHook threads.
    /// Used by the flow switcher to gate edge transitions on hold-key (§4.2).
    pub modifier_state: Arc<ModifierState>,

    pub config_change_listener: Mutex<Option<Box<dyn Fn(&Config, u64) + Send + Sync + 'static>>>,

    /// Handle used to push Actions Ring open/close signals to GUI clients.
    /// None until the daemon wires it in after starting the gRPC server.
    pub actions_ring_bc: Mutex<Option<crate::grpc::ActionsRingBroadcast>>,
    /// Monotonic id for the current ring session (stale close signals ignored).
    pub actions_ring_session: AtomicU64,
}

impl EngineInner {
    /// Get (creating if absent) the per-device state for `key`.
    pub fn device_state(&self, key: &DeviceKey) -> PerDeviceState {
        let mut map = self.per_device.lock_safe();
        map.entry(key.clone())
            .or_insert_with(PerDeviceState::default)
            .clone()
    }
}
