use actions_ring::ActionRingConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Profile {
    pub label: String,
    pub apps: Vec<String>,
    pub mappings: HashMap<String, String>,
    #[serde(default)]
    pub icon: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct FlowPeer {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub layout_x: i32,
    pub layout_y: i32,
    pub paired: bool,
    pub fingerprint: String,
    pub auto_reconnect: bool,
    #[serde(default)]
    pub channel_index: u8,
}

fn default_local_name() -> String {
    let name = if let Ok(hostname) = std::fs::read_to_string("/proc/sys/kernel/hostname") {
        hostname.trim().to_string()
    } else if let Ok(hostname) = std::env::var("HOSTNAME") {
        hostname.trim().to_string()
    } else {
        String::new()
    };
    if name.is_empty() {
        "Computer".to_string()
    } else {
        name
    }
}

fn default_edge_threshold() -> i32 {
    5
}

fn default_hold_ctrl_only() -> bool {
    false
}

fn default_handoff_timeout_ms() -> u64 {
    500
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Settings {
    pub start_minimized: bool,
    pub start_at_login: bool,
    pub hscroll_threshold: i32,
    pub invert_hscroll: bool,
    pub invert_vscroll: bool,
    pub dpi: i32,
    pub smart_shift_mode: String,
    pub smart_shift_enabled: bool,
    pub smart_shift_threshold: i32,
    pub gesture_threshold: i32,
    pub gesture_deadzone: i32,
    pub gesture_timeout_ms: u64,
    pub gesture_cooldown_ms: u64,
    pub appearance_mode: String,
    pub debug_mode: bool,
    pub device_layout_overrides: HashMap<String, serde_json::Value>,
    pub language: String,
    pub ignore_trackpad: bool,
    pub accent_color: String,
    pub install_updates: bool,
    // Flow settings
    pub flow_enabled: bool,
    pub flow_local_name: String,
    /// Set to true when the user has explicitly chosen a custom flow name.
    /// When false, the hostname is auto-populated on load.
    #[serde(default)]
    pub flow_name_user_set: bool,
    pub flow_peers: Vec<FlowPeer>,
    pub flow_screen_width: i32,
    pub flow_screen_height: i32,
    pub flow_hold_key: String,
    pub flow_mouse_mode: String,
    pub flow_keyboard_linking: bool,
    #[serde(default)]
    pub flow_local_channel_index: u8,
    #[serde(default = "default_edge_threshold")]
    pub flow_edge_threshold: i32,
    #[serde(default = "default_hold_ctrl_only")]
    pub flow_hold_ctrl_only: bool,
    #[serde(default = "default_handoff_timeout_ms")]
    pub flow_handoff_timeout_ms: u64,
    #[serde(default)]
    pub actions_ring: ActionRingConfig,
    /// Named multi-step macros (Smart Actions) reachable from a ring slot via
    /// the `macro:<name>` action. Each entry is a sequence of action ids.
    #[serde(default)]
    pub macros: HashMap<String, Vec<String>>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            start_minimized: true,
            start_at_login: false,
            hscroll_threshold: 1,
            invert_hscroll: false,
            invert_vscroll: false,
            dpi: 1000,
            smart_shift_mode: "ratchet".to_string(),
            smart_shift_enabled: false,
            smart_shift_threshold: 25,
            gesture_threshold: 50,
            gesture_deadzone: 40,
            gesture_timeout_ms: 3000,
            gesture_cooldown_ms: 500,
            appearance_mode: "system".to_string(),
            debug_mode: false,
            device_layout_overrides: HashMap::new(),
            language: "en".to_string(),
            ignore_trackpad: true,
            accent_color: "#8b5cf6".to_string(),
            install_updates: true,
            // Flow settings default
            flow_enabled: false,
            flow_local_name: default_local_name(),
            flow_name_user_set: false,
            flow_peers: vec![],
            flow_screen_width: 1920,
            flow_screen_height: 1080,
            flow_hold_key: "none".to_string(),
            flow_mouse_mode: "software".to_string(),
            flow_keyboard_linking: true,
            flow_local_channel_index: 0,
            flow_edge_threshold: 5,
            flow_hold_ctrl_only: false,
            flow_handoff_timeout_ms: 500,
            actions_ring: ActionRingConfig::default(),
            macros: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ProfileGroup {
    pub profiles: HashMap<String, Profile>,
}

/// Stable identity for a physical device. Keyed by HID serial when available
/// (the granular, per-device scope the user asked for), otherwise by the
/// layout key (e.g. `mx_master_3`) so an unnamed device still gets its own
/// profile store. Serial is preferred because two identical models on one
/// receiver must stay independent.
///
/// Serialized as `"serial|layout"` (a plain string) so it can be a JSON object
/// key — serde_json cannot key a map with a struct, which silently broke every
/// config save (`HashMap<DeviceKey, _>` would fail to serialize and the write
/// was dropped, so saves never persisted).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceKey {
    pub serial: String,
    pub layout: String,
}

impl Serialize for DeviceKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{}|{}", self.serial, self.layout))
    }
}

impl<'de> Deserialize<'de> for DeviceKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        let (serial, layout) = raw.split_once('|').unwrap_or(("", &raw));
        Ok(DeviceKey {
            serial: serial.to_string(),
            layout: layout.to_string(),
        })
    }
}

impl DeviceKey {
    pub fn new(serial: &str, layout: &str) -> Self {
        DeviceKey {
            serial: serial.to_string(),
            layout: layout.to_string(),
        }
    }

    /// Display label for the GUI.
    pub fn label(&self) -> String {
        if self.serial.is_empty() {
            self.layout.clone()
        } else {
            self.serial.clone()
        }
    }
}

impl Default for DeviceKey {
    fn default() -> Self {
        DeviceKey {
            serial: String::new(),
            layout: "generic".to_string(),
        }
    }
}

/// One device's complete profile store: a `global` (all-apps) profile plus
/// any number of per-app profiles. Replaces the old single shared
/// `profile_groups` so each mouse/keyboard resolves apps independently.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct DeviceProfiles {
    #[serde(default = "default_global_profile")]
    pub active_profile: String,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

fn default_global_profile() -> String {
    "global".to_string()
}

fn default_active_app_profile() -> String {
    "global".to_string()
}

fn default_global_profile_struct() -> Profile {
    let mut mappings = HashMap::new();
    mappings.insert("middle".to_string(), "none".to_string());
    mappings.insert("middle_gesture_enabled".to_string(), "true".to_string());
    mappings.insert("middle_gesture_left".to_string(), "none".to_string());
    mappings.insert("middle_gesture_right".to_string(), "none".to_string());
    mappings.insert("middle_gesture_up".to_string(), "none".to_string());
    mappings.insert("middle_gesture_down".to_string(), "none".to_string());
    mappings.insert("gesture".to_string(), "none".to_string());
    mappings.insert("gesture_enabled".to_string(), "true".to_string());
    mappings.insert("gesture_left".to_string(), "none".to_string());
    mappings.insert("gesture_right".to_string(), "none".to_string());
    mappings.insert("gesture_up".to_string(), "none".to_string());
    mappings.insert("gesture_down".to_string(), "none".to_string());
    mappings.insert("xbutton1".to_string(), "alt_tab".to_string());
    mappings.insert("xbutton1_gesture_enabled".to_string(), "true".to_string());
    mappings.insert("xbutton1_gesture_left".to_string(), "none".to_string());
    mappings.insert("xbutton1_gesture_right".to_string(), "none".to_string());
    mappings.insert("xbutton1_gesture_up".to_string(), "none".to_string());
    mappings.insert("xbutton1_gesture_down".to_string(), "none".to_string());
    mappings.insert("xbutton2".to_string(), "alt_tab".to_string());
    mappings.insert("xbutton2_gesture_enabled".to_string(), "true".to_string());
    mappings.insert("xbutton2_gesture_left".to_string(), "none".to_string());
    mappings.insert("xbutton2_gesture_right".to_string(), "none".to_string());
    mappings.insert("xbutton2_gesture_up".to_string(), "none".to_string());
    mappings.insert("xbutton2_gesture_down".to_string(), "none".to_string());
    mappings.insert("hscroll_left".to_string(), "browser_back".to_string());
    mappings.insert("hscroll_right".to_string(), "browser_forward".to_string());
    mappings.insert("mode_shift".to_string(), "switch_scroll_mode".to_string());
    Profile {
        label: "Default (All Apps)".to_string(),
        apps: vec![],
        mappings,
        icon: String::new(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub version: i32,
    /// Legacy single-group store, kept for migration only (see `load_raw`).
    #[serde(default)]
    pub active_group: String,
    /// Legacy shared active profile, kept for migration only.
    #[serde(default = "default_active_app_profile")]
    pub active_app_profile: String,
    #[serde(default)]
    pub profile_groups: HashMap<String, ProfileGroup>,
    /// New: per-device profile stores keyed by `DeviceKey`.
    #[serde(default)]
    pub devices: HashMap<DeviceKey, DeviceProfiles>,
    pub settings: Settings,
}

impl Default for Config {
    fn default() -> Self {
        let mut default_mappings = HashMap::new();
        default_mappings.insert("middle".to_string(), "none".to_string());
        default_mappings.insert("middle_gesture_enabled".to_string(), "true".to_string());
        default_mappings.insert("middle_gesture_left".to_string(), "none".to_string());
        default_mappings.insert("middle_gesture_right".to_string(), "none".to_string());
        default_mappings.insert("middle_gesture_up".to_string(), "none".to_string());
        default_mappings.insert("middle_gesture_down".to_string(), "none".to_string());

        default_mappings.insert("gesture".to_string(), "none".to_string());
        default_mappings.insert("gesture_enabled".to_string(), "true".to_string());
        default_mappings.insert("gesture_left".to_string(), "none".to_string());
        default_mappings.insert("gesture_right".to_string(), "none".to_string());
        default_mappings.insert("gesture_up".to_string(), "none".to_string());
        default_mappings.insert("gesture_down".to_string(), "none".to_string());

        default_mappings.insert("xbutton1".to_string(), "alt_tab".to_string());
        default_mappings.insert("xbutton1_gesture_enabled".to_string(), "true".to_string());
        default_mappings.insert("xbutton1_gesture_left".to_string(), "none".to_string());
        default_mappings.insert("xbutton1_gesture_right".to_string(), "none".to_string());
        default_mappings.insert("xbutton1_gesture_up".to_string(), "none".to_string());
        default_mappings.insert("xbutton1_gesture_down".to_string(), "none".to_string());

        default_mappings.insert("xbutton2".to_string(), "alt_tab".to_string());
        default_mappings.insert("xbutton2_gesture_enabled".to_string(), "true".to_string());
        default_mappings.insert("xbutton2_gesture_left".to_string(), "none".to_string());
        default_mappings.insert("xbutton2_gesture_right".to_string(), "none".to_string());
        default_mappings.insert("xbutton2_gesture_up".to_string(), "none".to_string());
        default_mappings.insert("xbutton2_gesture_down".to_string(), "none".to_string());

        default_mappings.insert("hscroll_left".to_string(), "browser_back".to_string());
        default_mappings.insert("hscroll_right".to_string(), "browser_forward".to_string());
        default_mappings.insert("mode_shift".to_string(), "switch_scroll_mode".to_string());

        let global_profile = Profile {
            label: "Default (All Apps)".to_string(),
            apps: vec![],
            mappings: default_mappings,
            icon: String::new(),
        };

        let mut profiles = HashMap::new();
        profiles.insert("global".to_string(), global_profile);

        let mut devices = HashMap::new();
        devices.insert(
            DeviceKey::default(),
            DeviceProfiles {
                active_profile: "global".to_string(),
                profiles,
            },
        );

        Config {
            version: 14,
            active_group: "default".to_string(),
            active_app_profile: "global".to_string(),
            profile_groups: HashMap::new(),
            devices,
            settings: Settings::default(),
        }
    }
}

pub fn get_config_path() -> PathBuf {
    if cfg!(test) {
        // Unique per thread so concurrent tests don't clobber each other's
        // config file (they all load/save through this path).
        let mut path = std::env::temp_dir();
        path.push(format!(
            "mouser_test_config_{:?}.json",
            std::thread::current().id()
        ));
        path
    } else {
        let mut path = dirs::config_dir().unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        });
        path.push("Mouser");
        path.push("config.json");
        path
    }
}

pub fn get_log_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config")
    });
    path.push("Mouser");
    path.push("logs");
    path
}

pub fn get_grpc_socket_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config")
    });
    path.push("Mouser");
    let _ = std::fs::create_dir_all(&path);
    path.push("mouser_daemon.sock");
    path
}

impl Config {
    fn try_migrate_v11(content: &str) -> Option<Self> {
        // Tolerant legacy shape: the earliest configs had `active_profile` +
        // `profiles` at the top level and no `version`/`settings` keys at all
        // (pre-v11). Deserialize with defaults so a missing version/settings
        // doesn't reject the file and strand the user with an empty `devices`
        // map (which later panics the GUI's profile lookup).
        #[derive(serde::Deserialize)]
        struct OldConfig {
            #[serde(default)]
            version: i32,
            active_profile: String,
            profiles: HashMap<String, Profile>,
            #[serde(default)]
            settings: Settings,
        }

        if let Ok(old_cfg) = serde_json::from_str::<OldConfig>(content) {
            let mut profile_groups = HashMap::new();
            let mut profiles = old_cfg.profiles;
        if let Some(default_profile) = profiles
            .remove("default")
            .or_else(|| profiles.remove("Default"))
        {
            let mut global_profile = default_profile;
            global_profile.label = "Default (All Apps)".to_string();
            profiles.insert("global".to_string(), global_profile);
        }
            profile_groups.insert("default".to_string(), ProfileGroup { profiles });

            let active_app_profile = if old_cfg.active_profile.eq_ignore_ascii_case("default") {
                "global".to_string()
            } else {
                old_cfg.active_profile
            };

            let new_cfg = Config {
                version: 13,
                active_group: "default".to_string(),
                active_app_profile,
                profile_groups,
                devices: HashMap::new(),
                settings: old_cfg.settings,
            };
            let _ = new_cfg.save();
            log::info!("[Config] Successfully migrated v11 config to v13.");
            Some(new_cfg)
        } else {
            None
        }
    }

    /// Fold the legacy single-group `profile_groups["default"]` into the
    /// default `DeviceKey` so existing per-app profiles survive the v13→v14
    /// move to per-device stores. Idempotent: if `devices` already has content
    /// we keep it and just clear the legacy fields + bump the version.
    fn migrate_v13_to_v14(&mut self) {
        if self.devices.is_empty() {
            let mut profiles: HashMap<String, Profile> = HashMap::new();
            if let Some(group) = self.profile_groups.get("default") {
                for (name, profile) in &group.profiles {
                    profiles.insert(name.clone(), profile.clone());
                }
            }
            if !profiles.contains_key("global") {
                profiles.insert("global".to_string(), default_global_profile_struct());
            }
            let active = if self.active_app_profile.is_empty() {
                "global".to_string()
            } else {
                self.active_app_profile.clone()
            };
            self.devices.insert(
                DeviceKey::default(),
                DeviceProfiles {
                    active_profile: active,
                    profiles,
                },
            );
        }
        self.profile_groups.clear();
        self.active_group.clear();
        self.version = 14;
        let _ = self.save();
        log::info!("[Config] Migrated config to v14 (per-device profiles).");
    }

    fn load_raw() -> Self {
        let path = get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                match serde_json::from_str::<Config>(&content) {
                    Ok(mut cfg) => {
                        // A config with no `version` key deserializes to the
                        // `Default` version (14) via serde defaults — but it may
                        // still be a legacy file (top-level `active_profile` /
                        // `profiles`, or `profile_groups`) whose `devices` map is
                        // empty because it predates the per-device model. Detect
                        // that and run migration instead of trusting the default
                        // version, otherwise the user keeps an empty `devices`
                        // map and the GUI panics on a missing profile.
                        let is_legacy = cfg.devices.is_empty()
                            && (content.contains("\"profile_groups\"")
                                || content.contains("\"active_profile\"")
                                || content.contains("\"profiles\""));
                        if is_legacy {
                            if let Some(mut migrated) = Self::try_migrate_v11(&content) {
                                migrated.migrate_v13_to_v14();
                                return migrated;
                            }
                        }
                        if cfg.version >= 14 {
                            return cfg;
                        }
                        // v13 (pre-per-device) → v14: fold the old single
                        // `profile_groups` default group into the default device key.
                        if cfg.version == 13 {
                            cfg.migrate_v13_to_v14();
                            return cfg;
                        }
                        if cfg.version < 13 {
                            if let Some(mut migrated) = Self::try_migrate_v11(&content) {
                                migrated.migrate_v13_to_v14();
                                return migrated;
                            }
                        }
                        // Successfully deserialized but unexpected version — bump to 14.
                        log::warn!(
                            "[Config] Loaded config with version {}, upgrading version to 14.",
                            cfg.version
                        );
                        cfg.migrate_v13_to_v14();
                        return cfg;
                    }
                    Err(e) => {
                        log::warn!(
                            "[Config] Failed to parse config.json: {}. Attempting fallback.",
                            e
                        );
                        if let Some(mut migrated) = Self::try_migrate_v11(&content) {
                            migrated.migrate_v13_to_v14();
                            return migrated;
                        }
                        // Create a backup of the invalid configuration file to prevent permanent loss.
                        let mut backup_path = path.clone();
                        backup_path.set_extension("json.bak");
                        log::warn!("[Config] Copying corrupted config to {:?}", backup_path);
                        let _ = fs::copy(&path, backup_path);
                    }
                }
            }
        }
        Config::default()
    }

    pub fn load() -> Self {
        let mut cfg = Self::load_raw();
        // Only auto-populate the hostname when the user hasn't explicitly set a custom name.
        // The old approach compared against a hardcoded list of strings ("Computer 1", etc.)
        // which incorrectly overwrote legitimate hostnames (§7.4).
        if !cfg.settings.flow_name_user_set || cfg.settings.flow_local_name.is_empty() {
            cfg.settings.flow_local_name = default_local_name();
        }
        cfg.normalize_apps();
        // Persist any migration we just performed (load_raw mutates the version /
        // devices map but doesn't write) so the on-disk file matches the in-memory
        // state — otherwise a config that only ever loads (no device connect, no
        // explicit save) would stay at the legacy version forever.
        if cfg.version >= 14 {
            let _ = cfg.save();
        }
        cfg
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let mut cloned = self.clone();
        cloned.normalize_apps();
        let path = get_config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&cloned)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn normalize_apps(&mut self) {
        for dev in self.devices.values_mut() {
            for profile in dev.profiles.values_mut() {
                // Skip if already normalized (trim + lowercase already applied).
                if profile
                    .apps
                    .iter()
                    .all(|a| a.as_str() == a.trim() && a.as_str() == a.to_lowercase().as_str())
                {
                    continue;
                }
                profile.apps = profile
                    .apps
                    .iter()
                    .map(|a| a.trim().to_lowercase())
                    .collect();
            }
        }
        // Legacy store normalization (best-effort; migration moves these to `devices`).
        for group in self.profile_groups.values_mut() {
            for profile in group.profiles.values_mut() {
                profile.apps = profile
                    .apps
                    .iter()
                    .map(|a| a.trim().to_lowercase())
                    .collect();
            }
        }
    }

    /// Get (creating if absent) the profile store for a device. Callers that
    /// only want to *read* should use `get_device_profiles` and handle None.
    /// Writes must target the stable per-model anchor (`{serial:"", layout}`) so a
    /// save persists and mouse vs keyboard stay independent — so if the anchor
    /// isn't present yet we CREATE it here (this is what hotplug.rs triggers via
    /// `ensure_device_profiles` at device connect). Reads use `resolve_device_key`,
    /// which falls back to the catch-all `|generic` so a missing entry never panics.
    pub fn ensure_device_profiles(&mut self, key: &DeviceKey) -> &mut DeviceProfiles {
        let resolved = if self.devices.contains_key(key) {
            key.clone()
        } else if !key.layout.is_empty() {
            DeviceKey::new("", &key.layout)
        } else {
            DeviceKey::default()
        };
        self.devices
            .entry(resolved)
            .or_insert_with(|| DeviceProfiles {
                active_profile: "global".to_string(),
                profiles: {
                    let mut m = HashMap::new();
                    m.insert("global".to_string(), default_global_profile_struct());
                    m
                },
            })
    }

    pub fn get_device_profiles(&self, key: &DeviceKey) -> Option<&DeviceProfiles> {
        self.devices.get(key)
    }

    /// Pick the device key to use at runtime: prefer an exact serial match,
    /// then fall back to a serial-less (layout-only) key, then to the default
    /// catch-all key. Keeps per-serial independence while still resolving a
    /// profile for an unnamed device of the same model.
    pub fn resolve_device_key(&self, key: &DeviceKey) -> DeviceKey {
        if self.devices.contains_key(key) {
            return key.clone();
        }
        if !key.serial.is_empty() {
            let layout_only = DeviceKey::new("", &key.layout);
            if self.devices.contains_key(&layout_only) {
                return layout_only;
            }
        }
        if !key.layout.is_empty() {
            // Stable per-model anchor: mouse vs keyboard (and different models)
            // keep independent entries keyed by layout, and a volatile serial is
            // never the identity. Only use it once the device has been activated
            // (hotplug.rs creates the entry); otherwise fall back to the default
            // catch-all, which always carries "global" and prevents a panic.
            let layout_only = DeviceKey::new("", &key.layout);
            if self.devices.contains_key(&layout_only) {
                return layout_only;
            }
        }
        DeviceKey::default()
    }

    pub fn get_profile(&self, device: &DeviceKey, name: &str) -> Option<&Profile> {
        self.devices
            .get(&self.resolve_device_key(device))
            .and_then(|d| d.profiles.get(name))
    }

    pub fn get_resolved_mappings(
        &self,
        device: &DeviceKey,
        profile_name: &str,
    ) -> HashMap<String, String> {
        let mut resolved = HashMap::new();
        let dkey = self.resolve_device_key(device);
        if let Some(dev) = self.devices.get(&dkey) {
            if let Some(global_profile) = dev.profiles.get("global") {
                resolved = global_profile.mappings.clone();
            }
            if profile_name != "global" {
                if let Some(target_profile) = dev.profiles.get(profile_name) {
                    for (k, v) in &target_profile.mappings {
                        resolved.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        resolved
    }

    pub fn get_active_mappings(&self, device: &DeviceKey) -> HashMap<String, String> {
        let dkey = self.resolve_device_key(device);
        let active = self
            .devices
            .get(&dkey)
            .map(|d| d.active_profile.clone())
            .unwrap_or_else(|| "global".to_string());
        self.get_resolved_mappings(device, &active)
    }

    /// Loose identity matching between a foreground exe name and the
    /// application profiles registered for *this device* — so a `brave`
    /// profile on the mouse does not leak onto the keyboard (and vice versa).
    pub fn get_profile_for_app(&self, device: &DeviceKey, exe_name: &str) -> String {
        if exe_name.is_empty() {
            return "global".to_string();
        }
        let exe_lower = exe_name.trim().to_lowercase();
        let dkey = self.resolve_device_key(device);
        if let Some(dev) = self.devices.get(&dkey) {
            for (pname, pdata) in &dev.profiles {
                if let Some(app) = pdata.apps.first() {
                    if execs_match(app, &exe_lower) {
                        return pname.clone();
                    }
                }
            }
        }
        "global".to_string()
    }

    // ── Legacy (pre-v14) API shims — retained so old call sites and tests
    // compile. They operate on the migrated `devices` default key. Remove once
    // all call sites migrate (ponytail: delete when no `profile_groups` users left).
    pub fn get_profile_legacy(&self, name: &str) -> Option<&Profile> {
        self.get_profile(&DeviceKey::default(), name)
    }

    pub fn get_resolved_mappings_legacy(&self, profile_name: &str) -> HashMap<String, String> {
        self.get_resolved_mappings(&DeviceKey::default(), profile_name)
    }

    pub fn get_active_mappings_legacy(&self) -> HashMap<String, String> {
        self.get_active_mappings(&DeviceKey::default())
    }

    pub fn get_profile_for_app_legacy(&self, exe_name: &str) -> String {
        self.get_profile_for_app(&DeviceKey::default(), exe_name)
    }
}

/// Strip release-channel suffixes distros commonly append to the real exec
/// name (e.g. `brave-browser-stable` → `brave-browser`).
pub fn normalize_exec(exec: &str) -> String {
    let lower = exec.to_lowercase();
    for suffix in ["-stable", "-beta", "-dev", "-nightly", "-unstable", "-esr"] {
        if let Some(stripped) = lower.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }
    lower
}

/// True if two exec names likely refer to the same application. Desktop
/// files and `/proc/*/exe` frequently disagree on the exact binary name
/// (wrapper script vs real executable, version-tagged package name, etc.),
/// so exact string equality alone under-deduplicates.
pub fn execs_match(a: &str, b: &str) -> bool {
    let na = normalize_exec(a);
    let nb = normalize_exec(b);
    if na.is_empty() || nb.is_empty() {
        return false;
    }
    na == nb || na.contains(&nb) || nb.contains(&na)
}

/// Actions Ring configuration vocabulary.
///
/// The ring is a cursor-centred radial menu: a trigger opens an eight-position
/// layout and the engine/GUI executes the selected action. These types are
/// persisted in `config.json` and shared by the engine (trigger + execution)
/// and the GUI (renderer).
pub mod actions_ring {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    /// One of the eight fixed positions in an Actions Ring, clockwise from
    /// the top. Variant names are the persisted slot keys and must stay stable.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ActionRingSlot {
        /// Twelve o'clock.
        Top,
        /// Between top and right.
        TopRight,
        /// Three o'clock.
        Right,
        /// Between right and bottom.
        BottomRight,
        /// Six o'clock.
        Bottom,
        /// Between bottom and left.
        BottomLeft,
        /// Nine o'clock.
        Left,
        /// Between left and top.
        TopLeft,
    }

    impl ActionRingSlot {
        /// All ring positions in clockwise display order.
        pub const ALL: [Self; 8] = [
            Self::Top,
            Self::TopRight,
            Self::Right,
            Self::BottomRight,
            Self::Bottom,
            Self::BottomLeft,
            Self::Left,
            Self::TopLeft,
        ];

        /// Unit vector from the ring's centre to this slot, with positive Y
        /// pointing **down** — the screen convention egui (and GPUI) draw in.
        pub fn unit_offset(self) -> (f32, f32) {
            let d = std::f32::consts::FRAC_1_SQRT_2;
            match self {
                Self::Top => (0.0, -1.0),
                Self::TopRight => (d, -d),
                Self::Right => (1.0, 0.0),
                Self::BottomRight => (d, d),
                Self::Bottom => (0.0, 1.0),
                Self::BottomLeft => (-d, d),
                Self::Left => (-1.0, 0.0),
                Self::TopLeft => (-d, -d),
            }
        }

        /// Top-left corner at which to place this slot's `slot_size` box, on a
        /// square `canvas` whose ring has the given `radius`. All in the
        /// caller's own units.
        pub fn placement(self, canvas: f32, radius: f32, slot_size: f32) -> (f32, f32) {
            let (x, y) = self.unit_offset();
            (
                canvas / 2.0 + x * radius - slot_size / 2.0,
                canvas / 2.0 + y * radius - slot_size / 2.0,
            )
        }

        /// Stable string key used in config JSON and IPC payloads.
        pub fn as_str(&self) -> &'static str {
            match self {
                Self::Top => "Top",
                Self::TopRight => "TopRight",
                Self::Right => "Right",
                Self::BottomRight => "BottomRight",
                Self::Bottom => "Bottom",
                Self::BottomLeft => "BottomLeft",
                Self::Left => "Left",
                Self::TopLeft => "TopLeft",
            }
        }
    }

    /// The actions displayed at the eight fixed ring positions, keyed by slot.
    /// An absent key is an intentionally empty slot.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
    pub struct ActionRingLayout {
        #[serde(default)]
        pub slots: HashMap<ActionRingSlot, String>,
    }

    impl ActionRingLayout {
        /// Iterate populated slots in clockwise display order.
        pub fn ordered(&self) -> Vec<(ActionRingSlot, String)> {
            ActionRingSlot::ALL
                .iter()
                .filter_map(|&slot| self.slots.get(&slot).map(|a| (slot, a.clone())))
                .collect()
        }
    }

    /// Per-device (here, global) Actions Ring settings and application-specific
    /// layouts.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(default)]
    pub struct ActionRingConfig {
        /// Whether "Show Actions Ring" opens this ring.
        pub enabled: bool,
        /// Open the ring on a quick tap of the gesture button (no directional
        /// gesture). Mirrors Logitech Options+' hold-gesture-button-then-tap.
        #[serde(default = "default_true")]
        pub open_on_gesture_tap: bool,
        /// Layout used when the foreground application has no override.
        pub default: ActionRingLayout,
        /// Complete layout overrides keyed by foreground application identifier.
        #[serde(default)]
        pub per_app: HashMap<String, ActionRingLayout>,
        /// Named sub-layouts reachable from a slot via the `folder:<name>` action.
        #[serde(default)]
        pub folders: HashMap<String, ActionRingLayout>,
    }

    fn default_true() -> bool {
        true
    }

    impl ActionRingConfig {
        /// Resolve the complete layout for the foreground application.
        pub fn effective_layout(&self, app_id: Option<&str>) -> ActionRingLayout {
            app_id
                .and_then(|app| self.per_app.get(app))
                .cloned()
                .unwrap_or_else(|| self.default.clone())
        }
    }

    impl Default for ActionRingConfig {
        fn default() -> Self {
            use ActionRingSlot as Slot;
            let mut slots = HashMap::new();
            slots.insert(Slot::Top, "cut".to_string());
            slots.insert(Slot::TopRight, "copy".to_string());
            slots.insert(Slot::Right, "paste".to_string());
            slots.insert(Slot::BottomRight, "browser_forward".to_string());
            slots.insert(Slot::Bottom, "play_pause".to_string());
            slots.insert(Slot::BottomLeft, "browser_back".to_string());
            slots.insert(Slot::Left, "undo".to_string());
            slots.insert(Slot::TopLeft, "redo".to_string());
            Self {
                enabled: true,
                open_on_gesture_tap: true,
                default: ActionRingLayout { slots },
                per_app: HashMap::new(),
                folders: HashMap::new(),
            }
        }
    }

    #[cfg(test)]
    mod action_ring_tests {
        use super::*;

        #[test]
        fn slot_unit_offsets_point_screen_down() {
            // Top is straight up (negative Y), Bottom straight down (positive Y).
            assert_eq!(ActionRingSlot::Top.unit_offset(), (0.0, -1.0));
            assert_eq!(ActionRingSlot::Bottom.unit_offset(), (0.0, 1.0));
            assert_eq!(ActionRingSlot::Right.unit_offset(), (1.0, 0.0));
            assert!(ActionRingSlot::TopRight.unit_offset().1 < 0.0);
        }

        #[test]
        fn default_layout_has_eight_slots_in_order() {
            let cfg = ActionRingConfig::default();
            let ordered = cfg.default.ordered();
            assert_eq!(ordered.len(), 8);
            assert_eq!(ordered[0].0, ActionRingSlot::Top);
            assert_eq!(ordered[0].1, "cut");
            assert_eq!(ordered[7].1, "redo");
        }

        #[test]
        fn per_app_override_wins_over_default() {
            let mut cfg = ActionRingConfig::default();
            let mut slots = HashMap::new();
            slots.insert(ActionRingSlot::Top, "screenshot".to_string());
            cfg.per_app
                .insert("com.brave.Browser".to_string(), ActionRingLayout { slots });

            assert_eq!(
                cfg.effective_layout(Some("com.brave.Browser")).slots[&ActionRingSlot::Top],
                "screenshot"
            );
            // Unknown app falls back to default.
            assert_eq!(
                cfg.effective_layout(Some("com.unknown.App")).slots[&ActionRingSlot::Top],
                "cut"
            );
            // No app id → default.
            assert_eq!(
                cfg.effective_layout(None).slots[&ActionRingSlot::Top],
                "cut"
            );
        }

        #[test]
        fn layout_serializes_round_trip() {
            let cfg = ActionRingConfig::default();
            let json = serde_json::to_string(&cfg).unwrap();
            let back: ActionRingConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(back.default.ordered().len(), 8);
            assert!(cfg.enabled);
            // New flags survive a round trip with defaults.
            assert!(back.open_on_gesture_tap);
            assert!(back.folders.is_empty());
        }

        #[test]
        fn folder_config_round_trips() {
            let mut cfg = ActionRingConfig::default();
            let mut folder = ActionRingLayout::default();
            folder.slots.insert(ActionRingSlot::Top, "undo".to_string());
            folder
                .slots
                .insert(ActionRingSlot::Right, "redo".to_string());
            cfg.folders.insert("editing".to_string(), folder);

            let json = serde_json::to_string(&cfg).unwrap();
            let back: ActionRingConfig = serde_json::from_str(&json).unwrap();
            let editing = back.folders.get("editing").expect("folder preserved");
            assert_eq!(editing.slots[&ActionRingSlot::Top], "undo");
            assert_eq!(editing.slots[&ActionRingSlot::Right], "redo");
        }

        #[test]
        fn macro_action_config_round_trips() {
            let mut settings = super::super::Settings::default();
            settings.macros.insert(
                "screenshot".to_string(),
                vec!["win_d".to_string(), "screen_capture".to_string()],
            );
            let json = serde_json::to_string(&settings).unwrap();
            let back: super::super::Settings = serde_json::from_str(&json).unwrap();
            assert_eq!(
                back.macros["screenshot"],
                vec!["win_d".to_string(), "screen_capture".to_string()]
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_lookup_with_unique_app() {
        let mut config = Config::default();
        let custom_profile = Profile {
            label: "Brave Web Browser".to_string(),
            apps: vec!["brave-browser-stable".to_string()],
            mappings: HashMap::new(),
            icon: String::new(),
        };
        config
            .devices
            .entry(DeviceKey::default())
            .or_default()
            .profiles
            .insert("Brave Web Browser".to_string(), custom_profile);

        // Verify that it matches
        let device = DeviceKey::default();
        assert_eq!(
            config.get_profile_for_app(&device, "brave-browser-stable"),
            "Brave Web Browser"
        );
        assert_eq!(config.get_profile_for_app(&device, "firefox"), "global");
    }

    #[test]
    fn test_config_migration() {
        let old_json = r##"{
          "version": 11,
          "active_profile": "default",
          "profiles": {
            "default": {
              "label": "Default (All Apps)",
              "apps": [],
              "mappings": {
                "xbutton1": "alt_tab"
              }
            },
            "Brave Web Browser": {
              "label": "Brave Web Browser",
              "apps": ["brave-browser-stable"],
              "mappings": {
                "xbutton2": "play_pause"
              }
            }
          },
          "settings": {
            "start_minimized": true,
            "start_at_login": false,
            "hscroll_threshold": 1,
            "invert_hscroll": false,
            "invert_vscroll": false,
            "dpi": 1000,
            "smart_shift_mode": "ratchet",
            "smart_shift_enabled": false,
            "smart_shift_threshold": 25,
            "gesture_threshold": 50,
            "gesture_deadzone": 40,
            "gesture_timeout_ms": 3000,
            "gesture_cooldown_ms": 500,
            "appearance_mode": "system",
            "debug_mode": false,
            "device_layout_overrides": {},
            "language": "en",
            "ignore_trackpad": true,
            "accent_color": "#8b5cf6",
            "install_updates": true,
            "flow_enabled": false,
            "flow_local_name": "Computer 1",
            "flow_peers": [],
            "flow_screen_width": 1920,
            "flow_screen_height": 1080,
            "flow_hold_key": "ctrl",
            "flow_mouse_mode": "software",
            "flow_keyboard_linking": true
          }
        }"##;

        let path = get_config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, old_json);

        // Load the config (full v11 → v13 → v14 migration chain runs)
        let config = Config::load();

        // Verify version upgraded to 14 (per-device profiles)
        assert_eq!(config.version, 14);
        // Verify legacy group fields are cleared by migration
        assert!(config.active_group.is_empty());
        assert!(config.profile_groups.is_empty());
        // Verify active app profile mapped from default to global
        assert_eq!(config.active_app_profile, "global");

        // Verify profiles migrated into the default device key
        let device_profiles = config
            .devices
            .get(&DeviceKey::default())
            .expect("profiles migrated into default device key");
        let global_p = device_profiles.profiles.get("global").unwrap();
        assert_eq!(global_p.mappings.get("xbutton1").unwrap(), "alt_tab");

        let brave_p = device_profiles
            .profiles
            .get("Brave Web Browser")
            .unwrap();
        assert_eq!(brave_p.mappings.get("xbutton2").unwrap(), "play_pause");
    }

    // Ancient pre-version config: `active_profile` + `profiles` at top level
    // with NO `version`/`settings` keys (matches the user's real on-disk file).
    // Must migrate into a populated `devices` map, not strand an empty one.
    #[test]
    fn test_config_migration_ancient_v1() {
        let old_json = r##"{
          "active_profile": "Default",
          "profiles": {
            "Default": { "dpi": 8000, "smartshift": false, "mappings": {} },
            "Browser": { "dpi": 8000, "smartshift": true, "mappings": {} },
            "Gaming":  { "dpi": 2573, "smartshift": true, "mappings": {} }
          }
        }"##;

        let parsed: Config = serde_json::from_str(old_json).unwrap();
        // No version key → defaults to 14 (Config::default), devices empty.
        assert!(parsed.devices.is_empty(), "v0 parsed with empty devices");

        // Run the same path load_raw uses: v0 → try_migrate_v11 → v14.
        let migrated = Config::try_migrate_v11(old_json).expect("v1 legacy migrates");
        let migrated = {
            let mut m = migrated;
            m.migrate_v13_to_v14();
            m
        };
        assert_eq!(migrated.version, 14);
        let device_profiles = migrated
            .devices
            .get(&DeviceKey::default())
            .expect("v1 profiles migrated into devices");
        assert!(device_profiles.profiles.contains_key("global"));
        assert!(device_profiles.profiles.contains_key("Browser"));
        assert!(device_profiles.profiles.contains_key("Gaming"));
        // Default profile renamed to global.
        assert!(!device_profiles.profiles.contains_key("Default"));
    }

    #[test]
    fn test_config_with_missing_fields() {
        let incomplete_json = r##"{
          "version": 13,
          "active_group": "default",
          "active_app_profile": "global",
          "profile_groups": {
            "default": {
              "profiles": {
                "global": {
                  "label": "Default (All Apps)",
                  "mappings": {
                    "xbutton1": "alt_tab"
                  }
                }
              }
            }
          },
          "settings": {
            "start_minimized": true,
            "dpi": 1000
          }
        }"##;

        let parsed: Config = serde_json::from_str(incomplete_json).unwrap();

        // Check that specified fields are parsed correctly
        assert!(parsed.settings.start_minimized);
        assert_eq!(parsed.settings.dpi, 1000);

        // Check that missing fields got their default values
        assert!(!parsed.settings.start_at_login);
        assert_eq!(parsed.settings.language, "en");
        assert_eq!(parsed.settings.accent_color, "#8b5cf6");
        assert!(!parsed.settings.flow_enabled);
        assert!(parsed.settings.flow_keyboard_linking);

        // Note: deserializing v13 JSON directly (without Config::load / load_raw)
        // does not run migration, so legacy profile_groups survive untouched and
        // devices stays empty until load_raw folds them in.
        assert_eq!(parsed.version, 13);
        let group = parsed.profile_groups.get("default").unwrap();
        let global_p = group.profiles.get("global").unwrap();
        assert!(global_p.apps.is_empty());
        assert_eq!(global_p.mappings.get("xbutton1").unwrap(), "alt_tab");
    }

    #[test]
    fn test_get_profile_for_app_with_different_exec_variants() {
        let mut config = Config::default();
        let custom_profile = Profile {
            label: "Brave Web Browser".into(),
            apps: vec!["brave-browser-stable".into()],
            mappings: HashMap::new(),
            icon: "brave-browser".into(),
        };
        // register under the default device key
        config
            .devices
            .entry(DeviceKey::default())
            .or_default()
            .profiles
            .insert("Brave Web Browser".into(), custom_profile);

        let device = DeviceKey::default();
        // Foreground returns raw process basename (/proc/PID/exe → "brave"),
        // but the profile was registered from the .desktop file's Exec line
        // ("brave-browser-stable") — fuzzy match bridges the gap.
        assert_eq!(
            config.get_profile_for_app(&device, "brave"),
            "Brave Web Browser",
            "fuzzy match bridges process-vs-desktop exec gap"
        );
        assert_eq!(
            config.get_profile_for_app(&device, "BRAVE-BROWSER-STABLE"),
            "Brave Web Browser",
            "case-insensitive exact still works"
        );
        assert_eq!(config.get_profile_for_app(&device, "unknown"), "global");
        assert_eq!(config.get_profile_for_app(&device, ""), "global");
    }

    // Regression: a device key whose layout was derived from the Product ID
    // (e.g. HidppClient::get_layout_key preferring `layout_from_pid`) must still
    // resolve `global` when queried with a name-heuristic layout. This is the
    // exact mismatch that previously made `get_profile(.., "global")` return
    // None and panic the GUI's `unwrap()`.
    #[test]
    fn test_resolve_device_key_falls_back_across_layout_derivations() {
        let mut config = Config::default();
        // The daemon activates a serial-less layout anchor; the GUI queries with a
        // volatile serial but the same layout, so resolution must hit the anchor.
        let stored = DeviceKey::new("", "mx_master_3");
        config
            .devices
            .entry(stored)
            .or_default()
            .profiles
            .insert("global".to_string(), default_global_profile_struct());

        let queried = DeviceKey::new("MX Master 3 (volatile serial)", "mx_master_3");
        // resolve_device_key must find the layout anchor, so get_profile returns
        // the global profile rather than None.
        assert!(
            config.get_profile(&queried, "global").is_some(),
            "global must resolve by layout anchor regardless of serial"
        );
    }

    // Regression: writes must land on the same key reads resolve to, otherwise a
    // save under a volatile (serial-based) key is invisible after a reconnect and
    // "per-app profile not saving" surfaces. With the activated layout-only entry
    // present, ensure_device_profiles must target it, not a throwaway exact key.
    #[test]
    fn test_ensure_device_profiles_targets_resolved_layout_key() {
        let mut config = Config::default();
        config.devices.clear();
        let model_key = DeviceKey::new("", "mx_master_3");
        config
            .devices
            .entry(model_key.clone())
            .or_default()
            .profiles
            .insert("global".to_string(), default_global_profile_struct());

        // A write using the volatile full key must resolve to the model entry.
        let volatile = DeviceKey::new("*** MX Master 3 (ABC123)", "mx_master_3");
        {
            let dev = config.ensure_device_profiles(&volatile);
            dev.active_profile = "custom".to_string();
        }
        // Exactly one device entry, the layout-only one, carrying the write.
        assert_eq!(config.devices.len(), 1);
        assert_eq!(
            config.devices.get(&model_key).unwrap().active_profile,
            "custom"
        );
        // And reads resolve it back.
        assert!(config.get_profile(&volatile, "global").is_some());
    }

    // Regression: two different models keep independent per-app profile stores.
    #[test]
    fn test_different_models_stay_independent() {
        let mut config = Config::default();
        config.devices.clear();
        for layout in ["mx_master_3", "mx_mechanical"] {
            let k = DeviceKey::new("", layout);
            config
                .devices
                .entry(k)
                .or_default()
                .profiles
                .insert("global".to_string(), default_global_profile_struct());
        }
        let mouse = DeviceKey::new("any", "mx_master_3");
        let kb = DeviceKey::new("any", "mx_mechanical");
        config.ensure_device_profiles(&mouse).active_profile = "MouseApp".to_string();
        config.ensure_device_profiles(&kb).active_profile = "KbApp".to_string();

        assert_eq!(
            config.devices.get(&DeviceKey::new("", "mx_master_3")).unwrap().active_profile,
            "MouseApp"
        );
        assert_eq!(
            config.devices.get(&DeviceKey::new("", "mx_mechanical")).unwrap().active_profile,
            "KbApp"
        );
        assert_eq!(config.devices.len(), 2);
    }

    // Regression: a saved per-model profile must survive a config reload (the
    // original "save not persisting" bug). Exercises the real load→mutate→save→load
    // cycle through the test temp path.
    #[test]
    fn test_save_persists_across_reload_via_disk() {
        // Start from a v13 file so we also cover the migration on first load.
        let v13 = serde_json::json!({
            "version": 13,
            "active_group": "default",
            "active_app_profile": "global",
            "profile_groups": {
                "default": {
                    "profiles": {
                        "global": {"label": "Default (All Apps)", "apps": [], "mappings": {}, "icon": ""}
                    }
                }
            }
        });
        {
            let mut cfg = Config::default();
            cfg.devices.clear();
            let _ = std::fs::write(
                crate::config::get_config_path(),
                serde_json::to_string_pretty(&v13).unwrap(),
            );
        }

        // First load: migrates v13 -> v14, then a model entry is activated (as
        // hotplug.rs does on device connect) and a write is persisted.
        {
            let mut cfg = Config::load();
            let key = DeviceKey::new("", "mx_master_3");
            // Simulate device activation: ensure the model entry exists so writes
            // target it (mirrors hotplug.rs activation).
            cfg.ensure_device_profiles(&key);
            let dev = cfg.ensure_device_profiles(&key);
            dev.active_profile = "Brave".to_string();
            let _ = cfg.save();
        }

        // Second load: the activated model entry + its active_profile must persist.
        let reloaded = Config::load();
        let key = DeviceKey::new("", "mx_master_3");
        let dev = reloaded
            .devices
            .get(&key)
            .expect("model entry persisted across reload");
        assert_eq!(dev.active_profile, "Brave");
    }
}
