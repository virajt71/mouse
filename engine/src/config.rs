use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub label: String,
    pub apps: Vec<String>,
    pub mappings: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlowPeer {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub layout_x: i32, // -1: left, 1: right, 0: same
    pub layout_y: i32, // -1: top, 1: bottom, 0: same
    pub paired: bool,
    pub fingerprint: String,
    #[serde(default = "default_true")]
    pub auto_reconnect: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub flow_peers: Vec<FlowPeer>,
    pub flow_screen_width: i32,
    pub flow_screen_height: i32,
    pub flow_hold_key: String,
    pub flow_mouse_mode: String,
    pub flow_keyboard_linking: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProfileGroup {
    pub profiles: HashMap<String, Profile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub version: i32,
    pub active_group: String,
    pub active_app_profile: String,
    pub profile_groups: HashMap<String, ProfileGroup>,
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
        };

        let mut profiles = HashMap::new();
        profiles.insert("global".to_string(), global_profile);

        let mut profile_groups = HashMap::new();
        profile_groups.insert("default".to_string(), ProfileGroup { profiles });

        let settings = Settings {
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
            flow_local_name: "Computer 1".to_string(),
            flow_peers: vec![],
            flow_screen_width: 1920,
            flow_screen_height: 1080,
            flow_hold_key: "none".to_string(),
            flow_mouse_mode: "software".to_string(),
            flow_keyboard_linking: true,
        };

        Config {
            version: 12,
            active_group: "default".to_string(),
            active_app_profile: "global".to_string(),
            profile_groups,
            settings,
        }
    }
}

pub fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config")
    });
    path.push("Mouser");
    path.push("config.json");
    path
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



impl Config {
    pub fn load() -> Self {
        let path = get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<Config>(&content) {
                    if cfg.version == 12 {
                        return cfg;
                    }
                }
                
                // Fallback / migration from v11
                #[derive(serde::Deserialize)]
                struct OldConfig {
                    version: i32,
                    active_profile: String,
                    profiles: HashMap<String, Profile>,
                    settings: Settings,
                }

                if let Ok(old_cfg) = serde_json::from_str::<OldConfig>(&content) {
                    let mut profile_groups = HashMap::new();
                    let mut profiles = old_cfg.profiles;
                    if let Some(default_profile) = profiles.remove("default") {
                        let mut global_profile = default_profile;
                        global_profile.label = "Default (All Apps)".to_string();
                        profiles.insert("global".to_string(), global_profile);
                    }
                    profile_groups.insert("default".to_string(), ProfileGroup { profiles });

                    let active_app_profile = if old_cfg.active_profile == "default" {
                        "global".to_string()
                    } else {
                        old_cfg.active_profile
                    };

                    let new_cfg = Config {
                        version: 12,
                        active_group: "default".to_string(),
                        active_app_profile,
                        profile_groups,
                        settings: old_cfg.settings,
                    };
                    let _ = new_cfg.save();
                    return new_cfg;
                } else {
                    log::warn!("[Config] Failed to parse config.json, using defaults.");
                }
            }
        }
        Config::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = get_config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profile_groups.get(&self.active_group)
            .and_then(|g| g.profiles.get(name))
    }

    pub fn get_resolved_mappings(&self, profile_name: &str) -> HashMap<String, String> {
        let mut resolved = HashMap::new();
        if let Some(group) = self.profile_groups.get(&self.active_group) {
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

    pub fn get_profile_for_app(&self, exe_name: &str) -> String {
        if exe_name.is_empty() {
            return "global".to_string();
        }
        let exe_lower = exe_name.to_lowercase();
        if let Some(group) = self.profile_groups.get(&self.active_group) {
            for (pname, pdata) in &group.profiles {
                if let Some(app) = pdata.apps.first() {
                    if app.trim().to_lowercase() == exe_lower {
                        return pname.clone();
                    }
                }
            }
        }
        "global".to_string()
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
        };
        if let Some(group) = config.profile_groups.get_mut("default") {
            group.profiles.insert("Brave Web Browser".to_string(), custom_profile);
        }

        // Verify that it matches
        assert_eq!(config.get_profile_for_app("brave-browser-stable"), "Brave Web Browser");
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

        // Verify version upgraded to 12
        assert_eq!(config.version, 12);
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
}
