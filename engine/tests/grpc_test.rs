use std::time::Duration;
use tempfile::tempdir;
use serde_json;

#[test]
fn test_grpc_client_server_communication() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test_daemon.sock");
    let config_path = dir.path().join("config.json");
    std::env::set_var("MOUSER_CONFIG_PATH", &config_path);
    
    // Write a clean v13 config (to test migration)
    let v13_config = serde_json::json!({
        "version": 13,
        "active_group": "default",
        "active_app_profile": "global",
        "profile_groups": {
            "default": {
                "name": "default",
                "profiles": {
                    "global": {
                        "label": "Default (All Apps)",
                        "apps": [],
                        "mappings": {},
                        "icon": ""
                    }
                },
                "apps": []
            }
        },
        "settings": {}
    });
    let _ = std::fs::write(&config_path, v13_config.to_string());
    
    // Load it via Config::load()
    let loaded = mouser_engine::config::Config::load();
    eprintln!("[DEBUG] Config::load(): version={}, groups={:?}", 
        loaded.version, loaded.profile_groups.keys().collect::<Vec<_>>());
    assert_eq!(loaded.version, 14, "Config::load() should migrate v13 to v14");
    
    // Create engine
    let engine = mouser_engine::Engine::new();
    let (_config_bc, _device_state_bc) =
        mouser_engine::grpc::start_grpc_server(engine.clone(), &socket_path).unwrap();

    // Wait for the server
    let start = std::time::Instant::now();
    loop {
        if socket_path.exists() { break; }
        if start.elapsed() > Duration::from_secs(5) {
            panic!("gRPC server did not bind socket within 5s");
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    let client = mouser_engine::client::EngineClient::connect(&socket_path)
        .expect("Client should connect to gRPC server over UDS");

    // Get initial config
    let config = client.get_config();
    eprintln!("[DEBUG] client.get_config() #1: version={}, groups={:?}",
        config.version, config.profile_groups.keys().collect::<Vec<_>>());
    assert_eq!(config.version, 14);

    // Add a profile
    client.add_profile("TestApp");
    eprintln!("[DEBUG] client.add_profile(TestApp) returned");

    // Check disk
    let on_disk = std::fs::read_to_string(&config_path).unwrap();
    eprintln!("[DEBUG] On disk after add_profile: {}", &on_disk[..on_disk.len().min(400)]);
    
    // Give time for async processing
    std::thread::sleep(Duration::from_millis(100));

    // Get config again
    let fresh_config = client.get_config();
    eprintln!("[DEBUG] client.get_config() #2: version={}, groups={:?}",
        fresh_config.version, fresh_config.profile_groups.keys().collect::<Vec<_>>());
    
    let group = fresh_config
        .profile_groups
        .get(&fresh_config.active_group)
        .expect("default group should exist");
    
    let profile_keys: Vec<&String> = group.profiles.keys().collect();
    eprintln!("[DEBUG] Profile keys in default group: {:?}", profile_keys);
    
    assert!(
        group.profiles.contains_key("TestApp"),
        "TestApp profile should exist. Available: {:?}", profile_keys
    );
}
