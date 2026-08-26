use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_grpc_client_server_communication() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test_daemon.sock");

    let engine = mouser_engine::Engine::new();
    let (config_bc, device_state_bc) =
        mouser_engine::grpc::start_grpc_server(engine.clone(), &socket_path).unwrap();

    // Give server a moment to start listening
    std::thread::sleep(Duration::from_millis(100));

    let client = mouser_engine::client::EngineClient::connect(&socket_path)
        .expect("Client should connect to gRPC server over UDS");

    let config = client.get_config();
    assert_eq!(config.version, 13);

    // Test RPC call
    client.add_profile("TestApp");
    
    // Give daemon time to process
    std::thread::sleep(Duration::from_millis(50));

    let fresh_config = client.get_config();
    let group = fresh_config.profile_groups.get(&fresh_config.active_group).unwrap();
    assert!(group.profiles.contains_key("TestApp"), "TestApp profile should exist in config");
}
