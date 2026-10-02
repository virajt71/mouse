use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct RingLayout {
    #[serde(default)]
    pub primary: Vec<RingBubble>,
    #[serde(default)]
    pub folders: Vec<RingFolder>,
    #[serde(default)]
    pub auto_close: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct RingBubble {
    #[serde(default)]
    pub action_id: String,
    #[serde(default)]
    pub icon_name: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub kind: RingBubbleKind,
    #[serde(default)]
    pub adjustment_range: Option<RingAdjustmentRange>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct RingAdjustmentRange {
    #[serde(default)]
    pub min: i32,
    #[serde(default)]
    pub max: i32,
    #[serde(default)]
    pub step: i32,
    #[serde(default)]
    pub default: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum RingBubbleKind {
    Action { action_id: String },
    Folder { folder_id: String },
}

impl Default for RingBubbleKind {
    fn default() -> Self {
        RingBubbleKind::Action { action_id: String::new() }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct RingFolder {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub bubbles: Vec<RingBubble>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct Profile {
    pub label: String,
    pub apps: Vec<String>,
    pub mappings: HashMap<String, String>,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub ring_layout: Option<RingLayout>,
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
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ProfileGroup {
    pub profiles: HashMap<String, Profile>,
    pub name: String,
    pub apps: Vec<String>,
}

/// Device identity: serial (HID++ device name) + layout key (PID-based).
/// Two identical mice stay independent because their serials differ.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct DeviceKey {
    pub serial: String,
    pub layout: String,
}
impl DeviceKey {
    pub fn new(serial: &str, layout: &str) -> Self {
        Self { serial: serial.to_string(), layout: layout.to_string() }
    }
}
/// Per-device profile storage — each connected device gets its own profile map.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct DeviceProfiles {
    pub profiles: HashMap<String, Profile>,
    pub active_app_profile: String,
}
// Custom serde for DeviceKey so HashMap keys survive JSON roundtrip
// even when the map is empty (de/serialize as a map with string keys).
impl serde::Serialize for DeviceKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as "serial|layout" string so DeviceKey works both as a
        // struct field and as a HashMap key (JSON object keys must be strings).
        serializer.serialize_str(&format!("{}|{}", self.serial, self.layout))
    }
}
impl<'de> serde::Deserialize<'de> for DeviceKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(Visitor)
    }
}

struct Visitor;

impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = DeviceKey;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a DeviceKey string (\'serial|layout\') or struct")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // String format: "serial|layout"
        if let Some((serial, layout)) = value.split_once('|') {
            return Ok(DeviceKey {
                serial: serial.to_string(),
                layout: layout.to_string(),
            });
        }
        // If it's not a pipe-separated string, try parsing as a plain serial
        // (single-field legacy format or just the serial number)
        Err(serde::de::Error::custom(
            "DeviceKey string must be \'serial|layout\' format",
        ))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        // Struct format: {"serial": "...", "layout": "..."}
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Serial,
            Layout,
        }
        let mut serial = None;
        let mut layout = None;
        while let Some(key) = map.next_key()? {
            match key {
                Field::Serial => serial = Some(map.next_value()?),
                Field::Layout => layout = Some(map.next_value()?),
            }
        }
        let serial = serial.ok_or_else(|| serde::de::Error::missing_field("serial"))?;
        let layout = layout.ok_or_else(|| serde::de::Error::missing_field("layout"))?;
        Ok(DeviceKey { serial, layout })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub version: i32,
    pub active_group: String,
    pub active_app_profile: String,
    pub profile_groups: HashMap<String, ProfileGroup>,
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
            ring_layout: None,
        };

        let mut profiles = HashMap::new();
        profiles.insert("global".to_string(), global_profile);

        let mut profile_groups = HashMap::new();
        profile_groups.insert("default".to_string(), ProfileGroup { name: "default".to_string(), profiles, apps: vec![] });

        Config {
            version: 13,
            active_group: "default".to_string(),
            active_app_profile: "global".to_string(),
            profile_groups,
            devices: HashMap::new(),
            settings: Settings::default(),
        }
    }
}

pub fn get_config_path() -> PathBuf {
    if let Ok(p) = std::env::var("MOUSER_CONFIG_PATH") {
        return PathBuf::from(p);
    }
    if cfg!(test) {
        let mut path = std::env::temp_dir();
        path.push("mouser_test_config.json");
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
        #[derive(serde::Deserialize)]
        struct OldConfig {
            version: i32,
            active_profile: String,
            profiles: HashMap<String, Profile>,
            settings: Settings,
        }

        if let Ok(old_cfg) = serde_json::from_str::<OldConfig>(content) {
            let mut profile_groups = HashMap::new();
            let mut profiles = old_cfg.profiles;
            if let Some(default_profile) = profiles.remove("default") {
                let mut global_profile = default_profile;
                global_profile.label = "Default (All Apps)".to_string();
                profiles.insert("global".to_string(), global_profile);
            }
            profile_groups.insert("default".to_string(), ProfileGroup { name: "default".to_string(), profiles, apps: vec![] });

            let active_app_profile = if old_cfg.active_profile == "default" {
                "global".to_string()
            } else {
                old_cfg.active_profile
            };

            let new_cfg = Config {
                version: 14,
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

    fn load_raw() -> Self {
        let path = get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                match serde_json::from_str::<Config>(&content) {
                    Ok(mut cfg) => {
                        if cfg.version == 13 {
                            if cfg.devices.is_empty() {
                                log::info!("[Config] Migrating v13 config: populating devices from cache");
                                cfg.devices = Self::migrate_devices_from_cache();
                            }
                            cfg.version = 14;
                            let _ = cfg.save();
                            return cfg;
                        }
                        if cfg.version < 13 {
                            if let Some(migrated) = Self::try_migrate_v11(&content) {
                                return migrated;
                            }
                        }
                        // version >= 14 — already current, accept as-is
                        // (no migration needed, just save to ensure consistency)
                        let _ = cfg.save();
                        return cfg;
                    }
                    Err(e) => {
                        log::warn!(
                            "[Config] Failed to parse config.json: {}. Attempting fallback.",
                            e
                        );
                        if let Some(migrated) = Self::try_migrate_v11(&content) {
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
        cfg
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let mut cloned = self.clone();
        cloned.normalize_apps();
        cloned.ensure_valid();
        let path = get_config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        cloned.version = 14;
        let content = serde_json::to_string_pretty(&cloned)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn normalize_apps(&mut self) {
        for group in self.profile_groups.values_mut() {
            for profile in group.profiles.values_mut() {
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
    }

    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        let ag = self.get_active_group();
        self.profile_groups
            .get(ag)
            .and_then(|g| g.profiles.get(name))
            .or_else(|| {
                self.profile_groups
                    .get("default")
                    .and_then(|g| g.profiles.get(name))
            })
    }

    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut Profile> {
        let ag = self.active_group.clone();
        let ag = if ag.is_empty() { "default".to_string() } else { ag };
        let group = self.profile_groups.get_mut(&ag)?;
        group.profiles.get_mut(name)
    }

    pub fn get_resolved_mappings(&self, profile_name: &str) -> HashMap<String, String> {
        let mut resolved = HashMap::new();
        let group = self
            .profile_groups
            .get(self.get_active_group())
            .or_else(|| self.profile_groups.get("default"));
        if let Some(group) = group {
            if let Some(global_profile) = group.profiles.get("global") {
                resolved = global_profile.mappings.clone();
            }
            if profile_name != "global" {
                if let Some(target_profile) = group.profiles.get(profile_name) {
                    for (k, v) in &target_profile.mappings {
                        resolved.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        resolved
    }

    pub fn get_active_mappings(&self) -> HashMap<String, String> {
        self.get_resolved_mappings(&self.active_app_profile)
    }

    /// Loose identity matching between a foreground exe name and registered
    /// application profiles — analogous to appcatalog's multi-identity
    /// `Application.matches()` but minimal: normalizes release-channel
    /// suffixes and checks substring containment so that e.g. the running
    /// process `brave` still matches a profile registered under
    /// `brave-browser-stable`.
    pub fn get_profile_for_app(&self, exe_name: &str) -> String {
        if exe_name.is_empty() {
            return "global".to_string();
        }
        let exe_lower = exe_name.trim().to_lowercase();
        let group = self
            .profile_groups
            .get(self.get_active_group())
            .or_else(|| self.profile_groups.get("default"));
        if let Some(group) = group {
            for (pname, pdata) in &group.profiles {
                if let Some(app) = pdata.apps.first() {
                    if execs_match(app, &exe_lower) {
                        return pname.clone();
                    }
                }
            }
        }
        "global".to_string()
    }

    /// Sync live device list into `devices`. Called by Engine::update_devices().
    pub fn update_devices(&mut self, bt_paired: &[(String, String, bool)], hidpp_names: &[String]) {
        // Build a map of BT paired devices by MAC
        let mut bt_map: HashMap<String, (String, bool)> = HashMap::new();
        for (mac, name, connected) in bt_paired {
            bt_map.insert(mac.clone(), (name.clone(), *connected));
        }
        // Build set of HID++ device names
        let hidpp_set: std::collections::HashSet<String> = hidpp_names.iter().cloned().collect();
        // Remove devices that are no longer connected
        self.devices.retain(|k, _| {
            bt_map.contains_key(&k.serial) || hidpp_set.contains(&k.serial)
        });
        // Add/update BT paired devices
        for (mac, name, _) in bt_paired {
            let key = DeviceKey::new(mac, &Self::layout_from_name(name));
            match self.devices.entry(key) {
                std::collections::hash_map::Entry::Vacant(e) => {
                    let mut profiles = HashMap::new();
                    profiles.insert(
                        "global".to_string(),
                        Profile {
                            label: format!("Default ({})", name),
                            apps: vec![],
                            mappings: HashMap::new(),
                            icon: String::new(),
                            ring_layout: None,
                        },
                    );
                    e.insert(DeviceProfiles {
                        profiles,
                        active_app_profile: "global".to_string(),
                    });
                }
                std::collections::hash_map::Entry::Occupied(_) => {}
            }
        }
    }
    /// Migrate devices from Bluetooth cache on v13 to v14 upgrade.
    fn migrate_devices_from_cache() -> HashMap<DeviceKey, DeviceProfiles> {
        let cached = crate::cache::load_device_cache();
        let mut devices = HashMap::new();
        for (mac, name, _) in &cached {
            let layout = Self::layout_from_name(name);
            let key = DeviceKey::new(mac, &layout);
            let mut profiles = HashMap::new();
            profiles.insert(
                "global".to_string(),
                Profile {
                    label: "Default (All Apps)".to_string(),
                    apps: vec![],
                    mappings: HashMap::new(),
                    icon: String::new(),
                    ring_layout: None,
                },
            );
            devices.insert(
                key,
                DeviceProfiles {
                    profiles,
                    active_app_profile: "global".to_string(),
                },
            );
        }
        devices
    }
    /// Map a device name to a layout key (PID-based).
    /// ponytail: name-based heuristic; switch to actual PID bytes from HID++ report if
    /// devices with identical names but different layouts ever appear in the wild.
    fn layout_from_name(name: &str) -> String {
        let nl = name.to_lowercase();
        if nl.contains("mx master 3s") || nl.contains("mx master 3") {
            return "mx_master_3".into();
        }
        if nl.contains("mx master 2s") || nl.contains("mx master 2") {
            return "mx_master_2s".into();
        }
        if nl.contains("mx master") {
            return "mx_master".into();
        }
        if nl.contains("mx anywhere 3s") || nl.contains("mx anywhere 3") {
            return "mx_anywhere_3".into();
        }
        if nl.contains("mx anywhere 2s") || nl.contains("mx anywhere 2") {
            return "mx_anywhere_2s".into();
        }
        if nl.contains("mx anywhere") {
            return "mx_anywhere".into();
        }
        if nl.contains("mx vertical") {
            return "mx_vertical".into();
        }
        if nl.contains("mx ergo") {
            return "mx_ergo".into();
        }
        if nl.contains("mx keys") || nl.contains("mx mechanical") {
            return "mx_mechanical".into();
        }
        if nl.contains("craft") {
            return "craft".into();
        }
        if nl.contains("g502") {
            return "g502".into();
        }
        if nl.contains("g304") || nl.contains("g305") {
            return "g304".into();
        }
        if nl.contains("g604") {
            return "g604".into();
        }
        if nl.contains("g703") {
            return "g703".into();
        }
        if nl.contains("g900") {
            return "g900".into();
        }
        if nl.contains("g903") {
            return "g903".into();
        }
        if nl.contains("lift") {
            return "lift".into();
        }
        if nl.contains("pop") {
            return "pop".into();
        }
        if nl.contains("m720") {
            return "m720".into();
        }
        if nl.contains("m336") || nl.contains("m337") {
            return "m336".into();
        }
        if nl.contains("m510") {
            return "m510".into();
        }
        if nl.contains("m585") || nl.contains("m590") {
            return "m585".into();
        }
        if nl.contains("m705") {
            return "m705".into();
        }
        if nl.contains("k850") || nl.contains("k860") {
            return "k850".into();
        }
        if nl.contains("k480") {
            return "k480".into();
        }
        if nl.contains("k380") {
            return "k380".into();
        }
        if nl.contains("pangolin") || nl.contains("trackball") {
            return "trackball".into();
        }
        "generic".into()
    }
    /// Return the active group name, falling back to "default" if empty.
    pub fn get_active_group(&self) -> &str {
        if self.active_group.is_empty() {
            "default"
        } else {
            &self.active_group
        }
    }
    /// Ensure required fields have valid defaults after load/migration.
    pub fn ensure_valid(&mut self) {
        if self.active_group.is_empty() {
            self.active_group = "default".to_string();
        }
        if self.active_app_profile.is_empty() {
            self.active_app_profile = "global".to_string();
        }
        // Ensure the active group exists
        let group = self
            .profile_groups
            .entry(self.active_group.clone())
            .or_insert_with(|| ProfileGroup {
                name: self.active_group.clone(),
                profiles: HashMap::new(),
                apps: vec![],
            });
        // Ensure "global" profile exists in the active group
        match group.profiles.entry("global".to_string()) {
            std::collections::hash_map::Entry::Vacant(e) => {
                if let Some(def_group) = Config::default().profile_groups.get("default") {
                    if let Some(gp) = def_group.profiles.get("global") {
                        e.insert(gp.clone());
                    }
                }
            }
            std::collections::hash_map::Entry::Occupied(_) => {}
        }
    }

    /// Returns the ring layout for the active profile, if configured.
    pub fn active_profile_ring_layout(&self) -> Option<RingLayout> {
        self.profile_groups
            .get(&self.active_group)
            .and_then(|g| g.profiles.get(&self.active_app_profile))
            .and_then(|p| p.ring_layout.clone())
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
            ring_layout: None,
        };
        if let Some(group) = config.profile_groups.get_mut("default") {
            group
                .profiles
                .insert("Brave Web Browser".to_string(), custom_profile);
        }

        // Verify that it matches
        assert_eq!(
            config.get_profile_for_app("brave-browser-stable"),
            "Brave Web Browser"
        );
        assert_eq!(config.get_profile_for_app("firefox"), "global");
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

        // Load the config
        let config = Config::load();

        // Verify version upgraded to 13
        assert_eq!(config.version, 14);
        // Verify active group is default
        assert_eq!(config.active_group, "default");
        // Verify active app profile mapped from default to global
        assert_eq!(config.active_app_profile, "global");

        // Verify default group contains the profiles
        let group = config.profile_groups.get("default").unwrap();
        let global_p = group.profiles.get("global").unwrap();
        assert_eq!(global_p.mappings.get("xbutton1").unwrap(), "alt_tab");

        let brave_p = group.profiles.get("Brave Web Browser").unwrap();
        assert_eq!(brave_p.mappings.get("xbutton2").unwrap(), "play_pause");
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

        // Check profiles missing apps list got empty vec
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
            ring_layout: None,
        };
        if let Some(group) = config.profile_groups.get_mut("default") {
            group
                .profiles
                .insert("Brave Web Browser".into(), custom_profile);
        }

        // Foreground returns raw process basename (/proc/PID/exe → "brave"),
        // but the profile was registered from the .desktop file's Exec line
        // ("brave-browser-stable") — fuzzy match bridges the gap.
        assert_eq!(
            config.get_profile_for_app("brave"),
            "Brave Web Browser",
            "fuzzy match bridges process-vs-desktop exec gap"
        );
        assert_eq!(
            config.get_profile_for_app("BRAVE-BROWSER-STABLE"),
            "Brave Web Browser",
            "case-insensitive exact still works"
        );
        assert_eq!(config.get_profile_for_app("unknown"), "global");
        assert_eq!(config.get_profile_for_app(""), "global");
    }

    #[test]
    fn test_layout_from_name() {
        assert_eq!(Config::layout_from_name("MX Master 3"), "mx_master_3");
        assert_eq!(Config::layout_from_name("mx master 3s"), "mx_master_3");
        assert_eq!(Config::layout_from_name("MX Anywhere 3"), "mx_anywhere_3");
        assert_eq!(Config::layout_from_name("Logitech Lift"), "lift");
        assert_eq!(Config::layout_from_name("MX Keys"), "mx_mechanical");
        assert_eq!(Config::layout_from_name("G502"), "g502");
        assert_eq!(Config::layout_from_name("Unknown Device 123"), "generic");
    }
    #[test]
    fn test_migrate_devices_from_cache() {
        // Create a temp CSV cache with known devices
        let tmp = std::env::temp_dir().join("mouser_test_cache_devices.csv");
        let _ = std::fs::write(
            &tmp,
            "AA:BB:CC:DD:EE:01,MX Master 3,true\nAA:BB:CC:DD:EE:02,Logitech Lift,false\nAA:BB:CC:DD:EE:03,Unknown Device, true",
        );
        // Override MOUSER_DEVICE_CACHE_PATH
        let old_env = std::env::var("MOUSER_DEVICE_CACHE_PATH").ok();
        std::env::set_var("MOUSER_DEVICE_CACHE_PATH", &tmp);
        // Create a v13 config that will trigger migration
        let v13 = r#"{"version":13,"active_group":"default","active_app_profile":"global","profile_groups":{},"settings":{}}"#;
        let tmp_cfg = std::env::temp_dir().join(format!("mouser_test_config_migrate_{}.json", std::process::id()));
        let _ = std::fs::write(&tmp_cfg, v13);
        let old_cfg_env = std::env::var("MOUSER_CONFIG_PATH").ok();
        std::env::set_var("MOUSER_CONFIG_PATH", &tmp_cfg);
        let cfg = Config::load();
        assert_eq!(cfg.version, 14);
        assert_eq!(cfg.devices.len(), 3);
        assert!(cfg.devices.contains_key(&DeviceKey::new("AA:BB:CC:DD:EE:01", "mx_master_3")));
        assert!(cfg.devices.contains_key(&DeviceKey::new("AA:BB:CC:DD:EE:02", "lift")));
        assert!(cfg.devices.contains_key(&DeviceKey::new("AA:BB:CC:DD:EE:03", "generic")));
        for (_, dp) in cfg.devices.iter() {
            assert!(dp.profiles.contains_key("global"));
            assert_eq!(dp.active_app_profile, "global");
        }
        // Cleanup
        if let Some(v) = old_env {
            std::env::set_var("MOUSER_DEVICE_CACHE_PATH", v);
        } else {
            std::env::remove_var("MOUSER_DEVICE_CACHE_PATH");
        }
        if let Some(v) = old_cfg_env {
            std::env::set_var("MOUSER_CONFIG_PATH", v);
        } else {
            std::env::remove_var("MOUSER_CONFIG_PATH");
        }
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&tmp_cfg);
    }
}
