use mouser_engine::lock_ext::MutexExt;
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

    let state = engine.inner.gesture_state.lock_safe();
    assert_eq!(state.delta_x, 0.0);
    assert_eq!(state.delta_y, 0.0);
}

// §6.2 — get_profile_for_app edge cases
#[test]
fn test_profile_matching_empty_exe() {
    let config = Config::default();
    // Empty string should always fall back to "global"
    assert_eq!(config.get_profile_for_app(""), "global");
}

#[test]
fn test_profile_matching_whitespace_exe() {
    let config = Config::default();
    // Whitespace-only exe should fall back to "global"
    assert_eq!(config.get_profile_for_app("   "), "global");
}

#[test]
fn test_profile_matching_mixed_case() {
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
        "vscode_profile".to_string(),
        Profile {
            label: "VS Code".to_string(),
            apps: vec!["code".to_string()],
            mappings: HashMap::new(),
        },
    );
    config
        .profile_groups
        .insert("default".to_string(), ProfileGroup { profiles });
    config.normalize_apps();

    // Mixed case should normalize to lowercase before matching
    assert_eq!(config.get_profile_for_app("CODE"), "vscode_profile");
    assert_eq!(config.get_profile_for_app("Code"), "vscode_profile");
    assert_eq!(config.get_profile_for_app("code"), "vscode_profile");
}

#[test]
fn test_profile_matching_no_match_returns_global() {
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
    config
        .profile_groups
        .insert("default".to_string(), ProfileGroup { profiles });
    config.normalize_apps();

    // Non-matching exe should return "global"
    assert_eq!(config.get_profile_for_app("gimp"), "global");
    assert_eq!(config.get_profile_for_app("terminal"), "global");
}

// §6.1 — Gesture button clear regression (§3.2 fix)
#[test]
fn test_gesture_button_cleared_after_up() {
    let engine = Engine::new();

    // Simulate an up event when gesture was never activated.
    // Before the §3.2 fix, this left state.button populated.
    engine.handle_gesture_up();

    let state = engine.inner.gesture_state.lock_safe();
    // button must be None after any up event, even on an idle engine
    assert!(
        state.button.is_none(),
        "gesture button should be None after handle_gesture_up, got {:?}",
        state.button
    );
}

#[test]
fn test_normalize_apps_idempotent() {
    let mut config = Config::default();
    let mut profiles = HashMap::new();
    profiles.insert(
        "global".to_string(),
        Profile {
            label: "Default".to_string(),
            apps: vec!["chrome".to_string(), "firefox".to_string()],
            mappings: HashMap::new(),
        },
    );
    config
        .profile_groups
        .insert("default".to_string(), ProfileGroup { profiles });

    // Already normalized — calling twice should be a no-op
    config.normalize_apps();
    config.normalize_apps();

    let group = config.profile_groups.get("default").unwrap();
    let global = group.profiles.get("global").unwrap();
    assert_eq!(global.apps, vec!["chrome", "firefox"]);
}
