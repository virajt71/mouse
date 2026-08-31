use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_grpc_client_server_communication() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test_daemon.sock");

    let engine = mouser_engine::Engine::new();
    let (worker_tx, _worker_rx) = std::sync::mpsc::channel();
    let (_config_bc, _device_state_bc, _actions_ring_bc) =
        mouser_engine::grpc::start_grpc_server(engine.clone(), &socket_path, worker_tx).unwrap();

    // Give server a moment to start listening
    std::thread::sleep(Duration::from_millis(100));

    let client = mouser_engine::client::EngineClient::connect(&socket_path)
        .expect("Client should connect to gRPC server over UDS");

    let config = client.get_config();
    assert_eq!(config.version, 14);
}
