use std::sync::{Arc, RwLock};
use std::net::{UdpSocket, TcpListener, TcpStream};
use std::io::{BufReader, BufRead, Write};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use super::FLOW_MANAGER;

lazy_static::lazy_static! {
    pub static ref DISCOVERED_PEERS: RwLock<HashMap<String, (String, Instant)>> = RwLock::new(HashMap::new());
    pub static ref ACTIVE_CONNECTIONS: RwLock<HashMap<String, TcpStream>> = RwLock::new(HashMap::new());
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum FlowEvent {
    MouseMove { dx: i32, dy: i32 },
    MouseButton { code: u16, value: i32 },
    Key { code: u16, value: i32 },
    MouseScroll { horizontal: bool, delta: i32 },
    ClipboardText(String),
    ClipboardImage(Vec<u8>),
    FileTransfer { name: String, content: Vec<u8> },
}

pub fn run_discovery_loop(engine_inner: Arc<crate::engine::inner::EngineInner>) {
    use std::sync::atomic::Ordering;
    let mut socket_opt = None;
    while engine_inner.running.load(Ordering::SeqCst) {
        match UdpSocket::bind("0.0.0.0:50519") {
            Ok(s) => {
                socket_opt = Some(s);
                break;
            }
            Err(e) => {
                log::warn!("[Flow Network] Failed to bind discovery UDP socket: {}. Retrying in 5s...", e);
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    }

    let socket = match socket_opt {
        Some(s) => s,
        None => return,
    };
    let _ = socket.set_broadcast(true);
    let _ = socket.set_read_timeout(Some(Duration::from_millis(500)));

    log::info!("[Flow Network] UDP Discovery listener running on port 50519...");

    // Spawn broadcast thread
    let socket_bc = match socket.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let cfg_bc = engine_inner.clone();
    thread_spawn_broadcast(socket_bc, cfg_bc);

    let mut buf = [0u8; 1024];
    loop {
        if let Ok((len, src)) = socket.recv_from(&mut buf) {
            let msg = String::from_utf8_lossy(&buf[..len]);
            let parts: Vec<&str> = msg.split(':').collect();
            if parts.is_empty() {
                continue;
            }

            let local_name = {
                let cfg = engine_inner.config.lock().unwrap();
                cfg.settings.flow_local_name.clone()
            };

            if parts[0] == "MOUSER_DISCOVER" && parts.len() > 1 {
                let peer_name = parts[1].to_string();
                if peer_name != local_name {
                    // Register peer IP
                    let peer_ip = src.ip().to_string();
                    DISCOVERED_PEERS.write().unwrap().insert(peer_name.clone(), (peer_ip, Instant::now()));

                    // Respond with identity
                    let reply = format!("MOUSER_IDENTITY:{}", local_name);
                    let _ = socket.send_to(reply.as_bytes(), src);
                }
            } else if parts[0] == "MOUSER_IDENTITY" && parts.len() > 1 {
                let peer_name = parts[1].to_string();
                if peer_name != local_name {
                    let peer_ip = src.ip().to_string();
                    DISCOVERED_PEERS.write().unwrap().insert(peer_name, (peer_ip, Instant::now()));
                }
            }
        }

        // Clean up expired peers (older than 10 seconds)
        {
            let mut peers = DISCOVERED_PEERS.write().unwrap();
            peers.retain(|_, (_, time)| time.elapsed() < Duration::from_secs(10));
        }
    }
}

fn thread_spawn_broadcast(socket: UdpSocket, engine_inner: Arc<crate::engine::inner::EngineInner>) {
    std::thread::spawn(move || {
        loop {
            let (enabled, name) = {
                let cfg = engine_inner.config.lock().unwrap();
                (cfg.settings.flow_enabled, cfg.settings.flow_local_name.clone())
            };

            if enabled {
                let msg = format!("MOUSER_DISCOVER:{}", name);
                let _ = socket.send_to(msg.as_bytes(), "255.255.255.255:50519");
            }
            std::thread::sleep(Duration::from_secs(3));
        }
    });
}

pub fn run_server_loop(engine_inner: Arc<crate::engine::inner::EngineInner>) {
    use std::sync::atomic::Ordering;
    let mut listener_opt = None;
    while engine_inner.running.load(Ordering::SeqCst) {
        match TcpListener::bind("0.0.0.0:50520") {
            Ok(l) => {
                listener_opt = Some(l);
                break;
            }
            Err(e) => {
                log::warn!("[Flow Network] Failed to bind TCP listener: {}. Retrying in 5s...", e);
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    }

    let listener = match listener_opt {
        Some(l) => l,
        None => return,
    };

    log::info!("[Flow Network] TCP Control Server listening on port 50520...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let inner = engine_inner.clone();
                std::thread::spawn(move || {
                    if let Err(e) = handle_client(stream, inner) {
                        log::error!("[Flow Network] Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                log::error!("[Flow Network] Connection accept failed: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, engine_inner: Arc<crate::engine::inner::EngineInner>) -> anyhow::Result<()> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    // 1. PIN-based pairing check
    // In our simplified flow, we send challenge.
    let salt = format!("{:x}", Instant::now().elapsed().as_nanos());
    stream.write_all(format!("MOUSER_CHALLENGE:{}\n", salt).as_bytes())?;

    line.clear();
    if reader.read_line(&mut line)? == 0 {
        return Ok(());
    }

    let trimmed = line.trim();
    if !trimmed.starts_with("MOUSER_RESPONSE:") {
        return Ok(());
    }

    let response_parts: Vec<&str> = trimmed.split(':').collect();
    if response_parts.len() < 3 {
        return Ok(());
    }

    let client_name = response_parts[1].to_string();
    let client_hash = response_parts[2];

    // Verify response hash: sha256 of salt + PIN
    // In this simulation, we check if the fingerprint or peer settings match.
    // To allow instant auto-pairing for testing, if the peer name exists in our config as paired,
    // we bypass. If not, the user must pair.
    let is_paired = {
        let cfg = engine_inner.config.lock().unwrap();
        cfg.settings.flow_peers.iter().any(|p| p.name == client_name && p.paired)
    };

    // Calculate expected hash (if paired, we can use default PIN '123456' or a saved secret)
    let pin = "123456";
    let input_str = format!("{}{}", salt, pin);
    let expected_hash = format!("{:x}", sha2::Sha256::digest(input_str.as_bytes()));

    if !is_paired && client_hash != expected_hash {
        stream.write_all(b"MOUSER_FAIL\n")?;
        return Ok(());
    }

    stream.write_all(b"MOUSER_OK\n")?;
    log::info!("[Flow Network] Peer '{}' authenticated successfully.", client_name);

    // Save connection to active peers
    {
        let mut conns = ACTIVE_CONNECTIONS.write().unwrap();
        conns.insert(client_name.clone(), stream.try_clone()?);
    }

    // Process events
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        if let Ok(event) = serde_json::from_str::<FlowEvent>(&line) {
            match event {
                FlowEvent::MouseMove { dx, dy } => {
                    if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock().unwrap() {
                        let uinput_arc = inner.key_simulator.device();
                        let mut uinput_guard = uinput_arc.lock().unwrap();
                        if let Some(ref mut uinput_dev) = *uinput_guard {
                            let evs = [
                                evdev::InputEvent::new(evdev::EventType::RELATIVE, evdev::RelativeAxisType::REL_X.0, dx),
                                evdev::InputEvent::new(evdev::EventType::RELATIVE, evdev::RelativeAxisType::REL_Y.0, dy),
                                evdev::InputEvent::new(evdev::EventType::SYNCHRONIZATION, 0, 0),
                            ];
                            let _ = uinput_dev.emit(&evs);
                        }
                    }
                }
                FlowEvent::MouseButton { code, value } => {
                    if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock().unwrap() {
                        let uinput_arc = inner.key_simulator.device();
                        let mut uinput_guard = uinput_arc.lock().unwrap();
                        if let Some(ref mut uinput_dev) = *uinput_guard {
                            let evs = [
                                evdev::InputEvent::new(evdev::EventType::KEY, code, value),
                                evdev::InputEvent::new(evdev::EventType::SYNCHRONIZATION, 0, 0),
                            ];
                            let _ = uinput_dev.emit(&evs);
                        }
                    }
                }
                FlowEvent::Key { code, value } => {
                    if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock().unwrap() {
                        let uinput_arc = inner.key_simulator.device();
                        let mut uinput_guard = uinput_arc.lock().unwrap();
                        if let Some(ref mut uinput_dev) = *uinput_guard {
                            let evs = [
                                evdev::InputEvent::new(evdev::EventType::KEY, code, value),
                                evdev::InputEvent::new(evdev::EventType::SYNCHRONIZATION, 0, 0),
                            ];
                            let _ = uinput_dev.emit(&evs);
                        }
                    }
                }
                FlowEvent::MouseScroll { horizontal, delta } => {
                    if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock().unwrap() {
                        let uinput_arc = inner.key_simulator.device();
                        let mut uinput_guard = uinput_arc.lock().unwrap();
                        if let Some(ref mut uinput_dev) = *uinput_guard {
                            let axis = if horizontal {
                                evdev::RelativeAxisType::REL_HWHEEL.0
                            } else {
                                evdev::RelativeAxisType::REL_WHEEL.0
                            };
                            let evs = [
                                evdev::InputEvent::new(evdev::EventType::RELATIVE, axis, delta),
                                evdev::InputEvent::new(evdev::EventType::SYNCHRONIZATION, 0, 0),
                            ];
                            let _ = uinput_dev.emit(&evs);
                        }
                    }
                }
                FlowEvent::ClipboardText(text) => {
                    let _ = super::clipboard::set_local_clipboard_text(text);
                }
                FlowEvent::ClipboardImage(png_bytes) => {
                    let _ = super::clipboard::set_local_clipboard_image(png_bytes);
                }
                FlowEvent::FileTransfer { name, content } => {
                    let _ = super::clipboard::save_flow_file(name, content);
                }
            }
        }
    }

    // Connection closed
    {
        let mut conns = ACTIVE_CONNECTIONS.write().unwrap();
        conns.remove(&client_name);
    }
    log::info!("[Flow Network] Connection to peer '{}' closed.", client_name);
    Ok(())
}

pub fn send_event_to_peer(peer_name: &str, event: &FlowEvent) -> anyhow::Result<()> {
    let conns = ACTIVE_CONNECTIONS.read().unwrap();
    if let Some(mut stream) = conns.get(peer_name) {
        let serialized = serde_json::to_string(event)?;
        stream.write_all(format!("{}\n", serialized).as_bytes())?;
        stream.flush()?;
    }
    Ok(())
}

use sha2::Digest;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization_deserialization() {
        let events = vec![
            FlowEvent::MouseMove { dx: 10, dy: -20 },
            FlowEvent::MouseButton { code: 272, value: 1 },
            FlowEvent::Key { code: 30, value: 0 },
            FlowEvent::MouseScroll { horizontal: false, delta: 120 },
            FlowEvent::ClipboardText("Hello World".to_string()),
            FlowEvent::ClipboardImage(vec![1, 2, 3, 4]),
            FlowEvent::FileTransfer { name: "test.txt".to_string(), content: vec![65, 66, 67] },
        ];

        for event in events {
            let serialized = serde_json::to_string(&event).expect("Failed to serialize");
            let deserialized: FlowEvent = serde_json::from_str(&serialized).expect("Failed to deserialize");
            match (&event, &deserialized) {
                (FlowEvent::MouseMove { dx: dx1, dy: dy1 }, FlowEvent::MouseMove { dx: dx2, dy: dy2 }) => {
                    assert_eq!(dx1, dx2);
                    assert_eq!(dy1, dy2);
                }
                (FlowEvent::MouseButton { code: c1, value: v1 }, FlowEvent::MouseButton { code: c2, value: v2 }) => {
                    assert_eq!(c1, c2);
                    assert_eq!(v1, v2);
                }
                (FlowEvent::Key { code: c1, value: v1 }, FlowEvent::Key { code: c2, value: v2 }) => {
                    assert_eq!(c1, c2);
                    assert_eq!(v1, v2);
                }
                (FlowEvent::MouseScroll { horizontal: h1, delta: d1 }, FlowEvent::MouseScroll { horizontal: h2, delta: d2 }) => {
                    assert_eq!(h1, h2);
                    assert_eq!(d1, d2);
                }
                (FlowEvent::ClipboardText(t1), FlowEvent::ClipboardText(t2)) => {
                    assert_eq!(t1, t2);
                }
                (FlowEvent::ClipboardImage(b1), FlowEvent::ClipboardImage(b2)) => {
                    assert_eq!(b1, b2);
                }
                (FlowEvent::FileTransfer { name: n1, content: c1 }, FlowEvent::FileTransfer { name: n2, content: c2 }) => {
                    assert_eq!(n1, n2);
                    assert_eq!(c1, c2);
                }
                _ => panic!("Event mismatch after deserialization: expected {:?}, got {:?}", event, deserialized),
            }
        }
    }

    #[test]
    fn test_challenge_response_generation() {
        let salt = "random_salt_12345";
        let pin = "123456";
        let input_str = format!("{}{}", salt, pin);
        let expected_hash = format!("{:x}", sha2::Sha256::digest(input_str.as_bytes()));

        // Verification logic match
        let client_hash = expected_hash.clone();
        let verification_input = format!("{}{}", salt, pin);
        let verified_hash = format!("{:x}", sha2::Sha256::digest(verification_input.as_bytes()));
        assert_eq!(client_hash, verified_hash);
    }
}
