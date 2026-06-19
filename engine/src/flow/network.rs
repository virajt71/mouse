use crate::lock_ext::MutexExt;
use super::FLOW_MANAGER;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub static DISCOVERED_PEERS: std::sync::LazyLock<RwLock<HashMap<String, (String, Instant)>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));
pub static ACTIVE_CONNECTIONS: std::sync::LazyLock<RwLock<HashMap<String, TcpStream>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));
pub static CONNECTING_PEERS: std::sync::LazyLock<RwLock<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| RwLock::new(std::collections::HashSet::new()));

pub static IS_SEARCHING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum FlowEvent {
    MouseMove { dx: i32, dy: i32 },
    MouseButton { code: u16, value: i32 },
    Key { code: u16, value: i32 },
    MouseScroll { horizontal: bool, delta: i32 },
    ClipboardText(String),
    ClipboardImage(Vec<u8>),
    FileTransfer { name: String, content: Vec<u8> },
    ReturnToLocal,
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
                log::warn!(
                    "[Flow Network] Failed to bind discovery UDP socket: {}. Retrying in 5s...",
                    e
                );
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
                let cfg = engine_inner.config.lock_safe();
                cfg.settings.flow_local_name.clone()
            };

            if parts[0] == "MOUSER_DISCOVER" && parts.len() > 1 {
                let peer_name = parts[1].to_string();
                if peer_name != local_name {
                    // Register peer IP
                    let peer_ip = src.ip().to_string();
                    DISCOVERED_PEERS
                        .write()
                        .unwrap()
                        .insert(peer_name.clone(), (peer_ip.clone(), Instant::now()));

                    // Respond with identity
                    let reply = format!("MOUSER_IDENTITY:{}", local_name);
                    let _ = socket.send_to(reply.as_bytes(), src);

                    trigger_auto_connect(&peer_name, &peer_ip, &engine_inner);
                }
            } else if parts[0] == "MOUSER_IDENTITY" && parts.len() > 1 {
                let peer_name = parts[1].to_string();
                if peer_name != local_name {
                    let peer_ip = src.ip().to_string();
                    DISCOVERED_PEERS
                        .write()
                        .unwrap()
                        .insert(peer_name.clone(), (peer_ip.clone(), Instant::now()));

                    trigger_auto_connect(&peer_name, &peer_ip, &engine_inner);
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
    use std::sync::atomic::Ordering;
    std::thread::spawn(move || loop {
        let (enabled, name) = {
            let cfg = engine_inner.config.lock_safe();
            (
                cfg.settings.flow_enabled,
                cfg.settings.flow_local_name.clone(),
            )
        };

        let searching = IS_SEARCHING.load(Ordering::SeqCst);

        if enabled || searching {
            let msg = format!("MOUSER_DISCOVER:{}", name);
            let _ = socket.send_to(msg.as_bytes(), "255.255.255.255:50519");
        }
        std::thread::sleep(Duration::from_secs(3));
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
                log::warn!(
                    "[Flow Network] Failed to bind TCP listener: {}. Retrying in 5s...",
                    e
                );
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

pub fn connect_to_peer(
    name: &str,
    ip: &str,
    port: u16,
    engine_inner: Arc<crate::engine::inner::EngineInner>,
) {
    // Skip if already connected or connecting
    {
        let conns = ACTIVE_CONNECTIONS.read().unwrap();
        if conns.contains_key(name) {
            return;
        }
    }
    {
        let mut connecting = CONNECTING_PEERS.write().unwrap();
        if !connecting.insert(name.to_string()) {
            return;
        }
    }

    struct Cleanup {
        name: String,
    }
    impl Drop for Cleanup {
        fn drop(&mut self) {
            CONNECTING_PEERS.write().unwrap().remove(&self.name);
        }
    }
    let _cleanup = Cleanup {
        name: name.to_string(),
    };

    let addr = format!("{}:{}", ip, port);
    log::info!(
        "[Flow Network] Attempting to connect to peer '{}' at {}",
        name,
        addr
    );
    let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(5))
    else {
        return;
    };

    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

    // Read challenge
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }

    let salt = line
        .trim()
        .strip_prefix("MOUSER_CHALLENGE:")
        .unwrap_or("")
        .to_string();

    // Get PIN from config (stored as fingerprint field for now)
    let pin = {
        let cfg = engine_inner.config.lock_safe();
        cfg.settings
            .flow_peers
            .iter()
            .find(|p| p.name == name)
            .map(|p| {
                if p.fingerprint.is_empty() {
                    "123456".to_string()
                } else {
                    p.fingerprint.clone()
                }
            })
            .unwrap_or("123456".to_string())
    };

    let hash = format!(
        "{:x}",
        sha2::Sha256::digest(format!("{}{}", salt, pin).as_bytes())
    );
    let local_name = engine_inner
        .config
        .lock()
        .unwrap()
        .settings
        .flow_local_name
        .clone();

    let _ = stream.write_all(format!("MOUSER_RESPONSE:{}:{}\n", local_name, hash).as_bytes());

    line.clear();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    if line.trim() != "MOUSER_OK" {
        return;
    }

    let _ = stream.set_read_timeout(None);

    log::info!("[Flow Network] Connected to peer '{}'", name);
    {
        let mut conns = ACTIVE_CONNECTIONS.write().unwrap();
        conns.insert(name.to_string(), stream.try_clone().unwrap());
    }

    // Spawn listener thread for the connected peer's events
    let stream_clone = stream.try_clone().unwrap();
    let name_clone = name.to_string();
    let inner_clone = engine_inner.clone();
    std::thread::spawn(move || {
        if let Err(e) = process_peer_events(stream_clone, name_clone.clone(), inner_clone) {
            log::error!(
                "[Flow Network] Error in process_peer_events for '{}': {}",
                name_clone,
                e
            );
        }
    });
}

fn trigger_auto_connect(
    peer_name: &str,
    peer_ip: &str,
    engine_inner: &Arc<crate::engine::inner::EngineInner>,
) {
    let (flow_enabled, is_paired, auto_reconnect) = {
        let cfg = engine_inner.config.lock_safe();
        if let Some(p) = cfg.settings.flow_peers.iter().find(|p| p.name == peer_name) {
            (cfg.settings.flow_enabled, p.paired, p.auto_reconnect)
        } else {
            (false, false, false)
        }
    };

    if flow_enabled && is_paired && auto_reconnect {
        let already_connected = {
            let conns = ACTIVE_CONNECTIONS.read().unwrap();
            conns.contains_key(peer_name)
        };
        let already_connecting = {
            let connecting = CONNECTING_PEERS.read().unwrap();
            connecting.contains(peer_name)
        };

        if !already_connected && !already_connecting {
            let ip_clone = peer_ip.to_string();
            let name_clone = peer_name.to_string();
            let inner_clone = engine_inner.clone();
            std::thread::spawn(move || {
                connect_to_peer(&name_clone, &ip_clone, 50520, inner_clone);
            });
        }
    }
}

fn handle_client(
    mut stream: TcpStream,
    engine_inner: Arc<crate::engine::inner::EngineInner>,
) -> anyhow::Result<()> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    // 1. PIN-based pairing check
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

    let peer_pin = {
        let cfg = engine_inner.config.lock_safe();
        cfg.settings
            .flow_peers
            .iter()
            .find(|p| p.name == client_name)
            .map(|p| {
                if p.fingerprint.is_empty() {
                    "123456".to_string()
                } else {
                    p.fingerprint.clone()
                }
            })
            .unwrap_or("123456".to_string())
    };

    let input_str = format!("{}{}", salt, peer_pin);
    let expected_hash = format!("{:x}", sha2::Sha256::digest(input_str.as_bytes()));

    if client_hash != expected_hash {
        stream.write_all(b"MOUSER_FAIL\n")?;
        return Ok(());
    }

    stream.write_all(b"MOUSER_OK\n")?;
    log::info!(
        "[Flow Network] Peer '{}' authenticated successfully.",
        client_name
    );

    {
        let mut conns = ACTIVE_CONNECTIONS.write().unwrap();
        conns.insert(client_name.clone(), stream.try_clone()?);
    }

    let _ = stream.set_read_timeout(None);
    process_peer_events(stream, client_name, engine_inner)
}

fn process_peer_events(
    stream: TcpStream,
    client_name: String,
    engine_inner: Arc<crate::engine::inner::EngineInner>,
) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        if let Ok(event) = serde_json::from_str::<FlowEvent>(&line) {
            match event {
                FlowEvent::MouseMove { dx, dy } => {
                    FLOW_MANAGER.set_current_controller(Some(client_name.clone()));
                    if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock_safe() {
                        let is_paired = {
                            let peers = FLOW_MANAGER.flow_peers.read().unwrap();
                            peers.iter().any(|p| p.name == client_name && p.paired)
                        };

                        if is_paired {
                            if let Some(target) = FLOW_MANAGER.handle_raw_motion(dx, dy) {
                                if target == client_name {
                                    log::info!(
                                        "[Flow Network] Edge transition back to controller '{}'",
                                        target
                                    );
                                    let _ =
                                        send_event_to_peer(&client_name, &FlowEvent::ReturnToLocal);
                                }
                            }
                        }

                        let uinput_arc = inner.key_simulator.device();
                        let mut uinput_guard = uinput_arc.lock_safe();
                        if let Some(ref mut uinput_dev) = *uinput_guard {
                            let evs = [
                                evdev::InputEvent::new(
                                    evdev::EventType::RELATIVE,
                                    evdev::RelativeAxisType::REL_X.0,
                                    dx,
                                ),
                                evdev::InputEvent::new(
                                    evdev::EventType::RELATIVE,
                                    evdev::RelativeAxisType::REL_Y.0,
                                    dy,
                                ),
                                evdev::InputEvent::new(evdev::EventType::SYNCHRONIZATION, 0, 0),
                            ];
                            let _ = uinput_dev.emit(&evs);
                        }
                    }
                }
                FlowEvent::MouseButton { code, value } => {
                    FLOW_MANAGER.set_current_controller(Some(client_name.clone()));
                    if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock_safe() {
                        let uinput_arc = inner.key_simulator.device();
                        let mut uinput_guard = uinput_arc.lock_safe();
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
                    FLOW_MANAGER.set_current_controller(Some(client_name.clone()));
                    if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock_safe() {
                        let uinput_arc = inner.key_simulator.device();
                        let mut uinput_guard = uinput_arc.lock_safe();
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
                    FLOW_MANAGER.set_current_controller(Some(client_name.clone()));
                    if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock_safe() {
                        let uinput_arc = inner.key_simulator.device();
                        let mut uinput_guard = uinput_arc.lock_safe();
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
                FlowEvent::ReturnToLocal => {
                    log::info!(
                        "[Flow Network] Received ReturnToLocal from peer '{}'",
                        client_name
                    );

                    let (lx, ly) = {
                        let cfg = engine_inner.config.lock_safe();
                        if let Some(peer) = cfg
                            .settings
                            .flow_peers
                            .iter()
                            .find(|p| p.name == client_name)
                        {
                            (peer.layout_x, peer.layout_y)
                        } else {
                            (0, 0)
                        }
                    };

                    FLOW_MANAGER.set_active_peer(None);

                    let sw = *FLOW_MANAGER.screen_width.read().unwrap();
                    let sh = *FLOW_MANAGER.screen_height.read().unwrap();

                    let target_x = if lx == 1 {
                        sw - 50
                    } else if lx == -1 {
                        50
                    } else {
                        sw / 2
                    };
                    let target_y = if ly == 1 {
                        sh - 50
                    } else if ly == -1 {
                        50
                    } else {
                        sh / 2
                    };

                    *FLOW_MANAGER.virtual_x.lock_safe() = target_x;
                    *FLOW_MANAGER.virtual_y.lock_safe() = target_y;

                    if let Ok((conn, screen_num)) =
                        x11rb::rust_connection::RustConnection::connect(None)
                    {
                        use x11rb::connection::Connection;
                        use x11rb::protocol::xproto::ConnectionExt;
                        let screen = &conn.setup().roots[screen_num];
                        let _ = conn.warp_pointer(
                            x11rb::NONE,
                            screen.root,
                            0,
                            0,
                            0,
                            0,
                            target_x as i16,
                            target_y as i16,
                        );
                        let _ = conn.flush();
                    }
                }
            }
        }
    }

    {
        let mut conns = ACTIVE_CONNECTIONS.write().unwrap();
        conns.remove(&client_name);
    }
    let active = FLOW_MANAGER.get_active_peer_name();
    if active.as_deref() == Some(&client_name) {
        FLOW_MANAGER.set_active_peer(None);
        log::info!(
            "[Flow Network] Peer '{}' disconnected — returning control to local.",
            client_name
        );
    }
    if FLOW_MANAGER.get_current_controller().as_ref() == Some(&client_name) {
        FLOW_MANAGER.set_current_controller(None);
    }
    log::info!(
        "[Flow Network] Connection to peer '{}' closed.",
        client_name
    );
    Ok(())
}

pub fn send_event_to_peer(peer_name: &str, event: &FlowEvent) -> anyhow::Result<()> {
    let stream = {
        let conns = ACTIVE_CONNECTIONS.read().unwrap();
        conns.get(peer_name).map(|s| s.try_clone())
    };
    if let Some(Ok(mut s)) = stream {
        let serialized = serde_json::to_string(event)?;
        s.write_all(format!("{}\n", serialized).as_bytes())?;
        s.flush()?;
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
            FlowEvent::MouseButton {
                code: 272,
                value: 1,
            },
            FlowEvent::Key { code: 30, value: 0 },
            FlowEvent::MouseScroll {
                horizontal: false,
                delta: 120,
            },
            FlowEvent::ClipboardText("Hello World".to_string()),
            FlowEvent::ClipboardImage(vec![1, 2, 3, 4]),
            FlowEvent::FileTransfer {
                name: "test.txt".to_string(),
                content: vec![65, 66, 67],
            },
            FlowEvent::ReturnToLocal,
        ];

        for event in events {
            let serialized = serde_json::to_string(&event).expect("Failed to serialize");
            let deserialized: FlowEvent =
                serde_json::from_str(&serialized).expect("Failed to deserialize");
            match (&event, &deserialized) {
                (
                    FlowEvent::MouseMove { dx: dx1, dy: dy1 },
                    FlowEvent::MouseMove { dx: dx2, dy: dy2 },
                ) => {
                    assert_eq!(dx1, dx2);
                    assert_eq!(dy1, dy2);
                }
                (
                    FlowEvent::MouseButton {
                        code: c1,
                        value: v1,
                    },
                    FlowEvent::MouseButton {
                        code: c2,
                        value: v2,
                    },
                ) => {
                    assert_eq!(c1, c2);
                    assert_eq!(v1, v2);
                }
                (
                    FlowEvent::Key {
                        code: c1,
                        value: v1,
                    },
                    FlowEvent::Key {
                        code: c2,
                        value: v2,
                    },
                ) => {
                    assert_eq!(c1, c2);
                    assert_eq!(v1, v2);
                }
                (
                    FlowEvent::MouseScroll {
                        horizontal: h1,
                        delta: d1,
                    },
                    FlowEvent::MouseScroll {
                        horizontal: h2,
                        delta: d2,
                    },
                ) => {
                    assert_eq!(h1, h2);
                    assert_eq!(d1, d2);
                }
                (FlowEvent::ClipboardText(t1), FlowEvent::ClipboardText(t2)) => {
                    assert_eq!(t1, t2);
                }
                (FlowEvent::ClipboardImage(b1), FlowEvent::ClipboardImage(b2)) => {
                    assert_eq!(b1, b2);
                }
                (
                    FlowEvent::FileTransfer {
                        name: n1,
                        content: c1,
                    },
                    FlowEvent::FileTransfer {
                        name: n2,
                        content: c2,
                    },
                ) => {
                    assert_eq!(n1, n2);
                    assert_eq!(c1, c2);
                }
                (FlowEvent::ReturnToLocal, FlowEvent::ReturnToLocal) => {}
                _ => panic!(
                    "Event mismatch after deserialization: expected {:?}, got {:?}",
                    event, deserialized
                ),
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
