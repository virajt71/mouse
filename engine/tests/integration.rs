use mouser_engine::config::{Config, Profile, ProfileGroup};
use mouser_engine::Engine;
use std::collections::HashMap;

#[test]
fn test_config_migration_roundtrip() {
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
        }
      },
      "settings": {
        "dpi": 1200
      }
    }"##;

    let parsed: Config = serde_json::from_str(old_json).unwrap();
    assert_eq!(parsed.settings.dpi, 1200);
}

#[test]
fn test_profile_matching_logic() {
    let mut config = Config::default();
    let mut profiles = HashMap::new();
    profiles.insert(
        "global".to_string(),
        Profile {
            label: "Default".to_string(),
            apps: vec![],
            mappings: HashMap::new(),
        },
    );
    profiles.insert(
        "chrome_profile".to_string(),
        Profile {
            label: "Chrome".to_string(),
            apps: vec!["chrome".to_string()],
            mappings: HashMap::new(),
        },
    );
    profiles.insert(
        "firefox_profile".to_string(),
        Profile {
            label: "Firefox".to_string(),
            apps: vec!["firefox".to_string()],
            mappings: HashMap::new(),
        },
    );

    config
        .profile_groups
        .insert("default".to_string(), ProfileGroup { profiles });
    config.normalize_apps();

    assert_eq!(config.get_profile_for_app("CHROME"), "chrome_profile");
    assert_eq!(config.get_profile_for_app("firefox "), "firefox_profile");
    assert_eq!(config.get_profile_for_app("other-app"), "global");
}

#[test]
fn test_gesture_state_transitions() {
    let engine = Engine::new();
    assert_eq!(engine.device_connected(), false);

    let state = engine.inner.gesture_state.lock().unwrap();
    assert_eq!(state.delta_x, 0.0);
    assert_eq!(state.delta_y, 0.0);
}
