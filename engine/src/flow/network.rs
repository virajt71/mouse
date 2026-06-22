use crate::lock_ext::MutexExt;
use super::FLOW_MANAGER;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use sha2::Digest;

pub static DISCOVERED_PEERS: std::sync::LazyLock<RwLock<HashMap<String, (String, u8, Instant)>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));
pub static ACTIVE_CONNECTIONS: std::sync::LazyLock<RwLock<HashMap<String, Arc<tokio::sync::Mutex<tokio::net::TcpStream>>>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));
pub static CONNECTING_PEERS: std::sync::LazyLock<RwLock<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| RwLock::new(std::collections::HashSet::new()));

pub static IS_SEARCHING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Per-IP connection rate limiter: (attempt_count, window_start).
/// Allows at most 5 new TCP connections per 60-second sliding window per source IP.
static CONNECTION_RATE_LIMITER: std::sync::LazyLock<
    std::sync::Mutex<HashMap<std::net::IpAddr, (u32, Instant)>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

const RATE_LIMIT_MAX_CONNS: u32 = 5;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

/// Returns `true` if the connection is within rate limits, `false` if it should be dropped.
fn check_rate_limit(ip: std::net::IpAddr) -> bool {
    let mut limiter = CONNECTION_RATE_LIMITER.lock().unwrap();
    let now = Instant::now();
    let entry = limiter.entry(ip).or_insert((0, now));
    if entry.1.elapsed() >= RATE_LIMIT_WINDOW {
        // Window expired — reset counter
        *entry = (1, now);
        true
    } else {
        entry.0 += 1;
        entry.0 <= RATE_LIMIT_MAX_CONNS
    }
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
    ReturnToLocal,
    FlowHandoffRequest {
        peer_id: String,
        entry_edge: crate::flow::edge::EdgeEvent,
        cursor_pos: (i32, i32),
    },
    FlowHandoffAck {
        peer_id: String,
    },
    FlowHandoffArm {
        peer_id: String,
        entry_edge: crate::flow::edge::EdgeEvent,
    },
    FlowHandoffDisarm {
        peer_id: String,
    },
    FlowReceptionAck {
        peer_id: String,
    },
}

pub struct NetworkPeerTransport;

#[async_trait::async_trait]
impl crate::flow::ports::PeerTransport for NetworkPeerTransport {
    async fn send_event(&self, peer_name: &str, event: &FlowEvent) -> anyhow::Result<()> {
        send_event_to_peer(peer_name, event).await
    }
}

pub async fn run_discovery_loop(engine_inner: Arc<crate::engine::inner::EngineInner>) {
    use std::sync::atomic::Ordering;
    let mut socket_opt = None;
    while engine_inner.running.load(Ordering::Acquire) {
        match tokio::net::UdpSocket::bind("0.0.0.0:50519").await {
            Ok(s) => {
                socket_opt = Some(s);
                break;
            }
            Err(e) => {
                log::warn!(
                    "[Flow Network] Failed to bind discovery UDP socket: {}. Retrying in 5s...",
                    e
                );
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    let socket = match socket_opt {
        Some(s) => Arc::new(s),
        None => return,
    };
    let _ = socket.set_broadcast(true);

    log::info!("[Flow Network] UDP Discovery listener running on port 50519...");

    // Spawn broadcast task
    let socket_bc = socket.clone();
    tokio::spawn(async move {
        loop {
            let enabled = FLOW_MANAGER.flow_enabled.load(Ordering::Relaxed);
            let searching = IS_SEARCHING.load(Ordering::Relaxed);

            if enabled || searching {
                let name = FLOW_MANAGER.flow_local_name.read().unwrap().clone();
                let channel = FLOW_MANAGER.flow_local_channel_index.load(Ordering::Relaxed);
                let msg = format!("MOUSER_DISCOVER:{}:{}", name, channel);
                let _ = socket_bc.send_to(msg.as_bytes(), "255.255.255.255:50519").await;
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });

    let mut buf = [0u8; 1024];
    let mut last_detected_peer_time: Option<Instant> = None;
    loop {
        if !engine_inner.running.load(Ordering::Acquire) {
            break;
        }

        tokio::select! {
            res = socket.recv_from(&mut buf) => {
                match res {
                    Ok((len, src)) => {
                        let msg = String::from_utf8_lossy(&buf[..len]);
                        let parts: Vec<&str> = msg.split(':').collect();
                        if parts.is_empty() {
                            continue;
                        }

                        let local_name = FLOW_MANAGER.flow_local_name.read().unwrap().clone();

                        if parts[0] == "MOUSER_DISCOVER" && parts.len() > 1 {
                            let peer_name = parts[1].to_string();
                            if peer_name != local_name {
                                let peer_channel = if parts.len() > 2 {
                                    parts[2].parse::<u8>().unwrap_or(0)
                                } else {
                                    0
                                };
                                let peer_ip = src.ip().to_string();
                                DISCOVERED_PEERS
                                    .write()
                                    .unwrap()
                                    .insert(peer_name.clone(), (peer_ip.clone(), peer_channel, Instant::now()));

                                let reply = format!("MOUSER_IDENTITY:{}:{}", local_name, {
                                    FLOW_MANAGER.flow_local_channel_index.load(Ordering::Relaxed)
                                });
                                let _ = socket.send_to(reply.as_bytes(), src).await;

                                {
                                    let mut cfg = engine_inner.config.lock_safe();
                                    if let Some(peer) = cfg.settings.flow_peers.iter_mut().find(|p| p.name == peer_name) {
                                        if peer.channel_index != peer_channel {
                                            peer.channel_index = peer_channel;
                                            let _ = cfg.save();
                                            engine_inner.config_generation.fetch_add(1, Ordering::SeqCst);
                                            if let Ok(lock) = engine_inner.config_change_listener.lock() {
                                                if let Some(ref callback) = *lock {
                                                    callback();
                                                }
                                            }
                                        }
                                    }
                                }

                                trigger_auto_connect(&peer_name, &peer_ip, &engine_inner).await;
                            }
                        } else if parts[0] == "MOUSER_IDENTITY" && parts.len() > 1 {
                            let peer_name = parts[1].to_string();
                            if peer_name != local_name {
                                let peer_channel = if parts.len() > 2 {
                                    parts[2].parse::<u8>().unwrap_or(0)
                                } else {
                                    0
                                };
                                let peer_ip = src.ip().to_string();
                                DISCOVERED_PEERS
                                    .write()
                                    .unwrap()
                                    .insert(peer_name.clone(), (peer_ip.clone(), peer_channel, Instant::now()));

                                {
                                    let mut cfg = engine_inner.config.lock_safe();
                                    if let Some(peer) = cfg.settings.flow_peers.iter_mut().find(|p| p.name == peer_name) {
                                        if peer.channel_index != peer_channel {
                                            peer.channel_index = peer_channel;
                                            let _ = cfg.save();
                                            engine_inner.config_generation.fetch_add(1, Ordering::SeqCst);
                                            if let Ok(lock) = engine_inner.config_change_listener.lock() {
                                                if let Some(ref callback) = *lock {
                                                    callback();
                                                }
                                            }
                                        }
                                    }
                                }

                                trigger_auto_connect(&peer_name, &peer_ip, &engine_inner).await;
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[Flow Network] UDP recv error: {}", e);
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(500)) => {
                let mut peers = DISCOVERED_PEERS.write().unwrap();
                peers.retain(|_, (_, _, time)| time.elapsed() < Duration::from_secs(10));
            }
        }

        let flow_enabled = FLOW_MANAGER.flow_enabled.load(Ordering::Relaxed);
        if flow_enabled {
            let has_active = !ACTIVE_CONNECTIONS.read().unwrap().is_empty();
            let has_discovered = !DISCOVERED_PEERS.read().unwrap().is_empty();
            if has_active || has_discovered {
                last_detected_peer_time = Some(Instant::now());
            } else {
                let last_time = last_detected_peer_time.get_or_insert_with(Instant::now);
                if last_time.elapsed() > Duration::from_secs(15) {
                    log::info!("[Flow Network] No other Flow devices detected for 15s. Automatically disabling Flow.");
                    {
                        let mut cfg = engine_inner.config.lock_safe();
                        cfg.settings.flow_enabled = false;
                        let _ = cfg.save();
                        FLOW_MANAGER.update_config(&cfg);
                        engine_inner.config_generation.fetch_add(1, Ordering::SeqCst);
                        if let Ok(lock) = engine_inner.config_change_listener.lock() {
                            if let Some(ref callback) = *lock {
                                callback();
                            }
                        }
                    }
                    last_detected_peer_time = None;
                }
            }
        } else {
            last_detected_peer_time = None;
        }
    }
}

pub async fn run_server_loop(engine_inner: Arc<crate::engine::inner::EngineInner>) {
    use std::sync::atomic::Ordering;
    let mut listener_opt = None;
    while engine_inner.running.load(Ordering::Acquire) {
        match tokio::net::TcpListener::bind("0.0.0.0:50520").await {
            Ok(l) => {
                listener_opt = Some(l);
                break;
            }
            Err(e) => {
                log::warn!(
                    "[Flow Network] Failed to bind TCP listener: {}. Retrying in 5s...",
                    e
                );
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    let listener = match listener_opt {
        Some(l) => l,
        None => return,
    };

    log::info!("[Flow Network] TCP Control Server listening on port 50520...");

    loop {
        if !engine_inner.running.load(Ordering::Acquire) {
            break;
        }

        tokio::select! {
            res = listener.accept() => {
                match res {
                    Ok((stream, peer_addr)) => {
                        let peer_ip = peer_addr.ip();
                        if !check_rate_limit(peer_ip) {
                            log::warn!(
                                "[Flow Network] Rate limit exceeded for {}. Dropping connection.",
                                peer_ip
                            );
                            drop(stream);
                        } else {
                            let inner = engine_inner.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_client(stream, inner).await {
                                    log::error!("[Flow Network] Error handling client: {}", e);
                                }
                            });
                        }
                    }
                    Err(e) => {
                        log::error!("[Flow Network] Connection accept failed: {}", e);
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(500)) => {}
        }
    }
}

pub async fn connect_to_peer(
    name: &str,
    ip: &str,
    port: u16,
    engine_inner: Arc<crate::engine::inner::EngineInner>,
) {
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

    let stream_res = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect(&addr),
    ).await;

    let mut stream = match stream_res {
        Ok(Ok(s)) => s,
        _ => return,
    };

    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let mut line = String::new();
    {
        let mut reader = BufReader::new(&mut stream);
        if reader.read_line(&mut line).await.is_err() {
            return;
        }
    }

    let salt = line
        .trim()
        .strip_prefix("MOUSER_CHALLENGE:")
        .unwrap_or("")
        .to_string();

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

    if stream.write_all(format!("MOUSER_RESPONSE:{}:{}\n", local_name, hash).as_bytes()).await.is_err() {
        return;
    }

    line.clear();
    {
        let mut reader = BufReader::new(&mut stream);
        if reader.read_line(&mut line).await.is_err() {
            return;
        }
    }
    if line.trim() != "MOUSER_OK" {
        return;
    }

    log::info!("[Flow Network] Connected to peer '{}'", name);
    let stream_arc = Arc::new(tokio::sync::Mutex::new(stream));
    {
        let mut conns = ACTIVE_CONNECTIONS.write().unwrap();
        conns.insert(name.to_string(), stream_arc.clone());
    }

    let name_clone = name.to_string();
    let inner_clone = engine_inner.clone();
    tokio::spawn(async move {
        if let Err(e) = process_peer_events(stream_arc, name_clone.clone(), inner_clone).await {
            log::error!(
                "[Flow Network] Error in process_peer_events for '{}': {}",
                name_clone,
                e
            );
        }
    });
}

async fn trigger_auto_connect(
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
            tokio::spawn(async move {
                connect_to_peer(&name_clone, &ip_clone, 50520, inner_clone).await;
            });
        }
    }
}

async fn handle_client(
    mut stream: tokio::net::TcpStream,
    engine_inner: Arc<crate::engine::inner::EngineInner>,
) -> anyhow::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let salt = format!("{:x}", Instant::now().elapsed().as_nanos());
    let write_res = tokio::time::timeout(
        Duration::from_secs(5),
        stream.write_all(format!("MOUSER_CHALLENGE:{}\n", salt).as_bytes())
    ).await;

    if write_res.is_err() || write_res.unwrap().is_err() {
        return Ok(());
    }

    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();

    let read_res = tokio::time::timeout(
        Duration::from_secs(5),
        reader.read_line(&mut line)
    ).await;

    if read_res.is_err() || read_res.unwrap().unwrap_or(0) == 0 {
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
        let _ = stream.write_all(b"MOUSER_FAIL\n").await;
        return Ok(());
    }

    stream.write_all(b"MOUSER_OK\n").await?;
    log::info!(
        "[Flow Network] Peer '{}' authenticated successfully.",
        client_name
    );

    let stream_arc = Arc::new(tokio::sync::Mutex::new(stream));
    {
        let mut conns = ACTIVE_CONNECTIONS.write().unwrap();
        conns.insert(client_name.clone(), stream_arc.clone());
    }

    process_peer_events(stream_arc, client_name, engine_inner).await
}

async fn process_peer_events(
    stream: Arc<tokio::sync::Mutex<tokio::net::TcpStream>>,
    client_name: String,
    engine_inner: Arc<crate::engine::inner::EngineInner>,
) -> anyhow::Result<()> {
    use tokio::io::AsyncBufReadExt;

    let mut line = String::new();
    loop {
        line.clear();
        let read_bytes = {
            let mut guard = stream.lock().await;
            let mut reader = tokio::io::BufReader::new(&mut *guard);
            reader.read_line(&mut line).await?
        };

        if read_bytes == 0 {
            break;
        }

        if let Ok(event) = serde_json::from_str::<FlowEvent>(&line) {
            match event {
                FlowEvent::MouseMove { dx, dy } => {
                    FLOW_MANAGER.set_current_controller(Some(client_name.clone()));
                    let mut should_return = false;
                    if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock_safe() {
                        let is_paired = {
                            let peers = FLOW_MANAGER.flow_peers.read().unwrap();
                            peers.iter().any(|p| p.name == client_name && p.paired)
                        };

                        if is_paired {
                            if let crate::flow::CursorZone::FireZone(edge_ev) = FLOW_MANAGER.handle_raw_motion(dx, dy) {
                                if let Some(peer) = crate::flow::topology::resolve_peer(edge_ev) {
                                    if peer.peer_id == client_name {
                                        log::info!(
                                            "[Flow Network] Edge transition back to controller '{}'",
                                            client_name
                                        );
                                        should_return = true;
                                    }
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
                    if should_return {
                        let _ = send_event_to_peer(&client_name, &FlowEvent::ReturnToLocal).await;
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
                FlowEvent::FlowHandoffRequest { peer_id, entry_edge, cursor_pos } => {
                    crate::flow::handoff::handle_handoff_request(peer_id, entry_edge, cursor_pos, &client_name);
                }
                FlowEvent::FlowHandoffAck { peer_id } => {
                    crate::flow::handoff::handle_handoff_ack(peer_id);
                }
                FlowEvent::FlowHandoffArm { peer_id, entry_edge } => {
                    crate::flow::handoff::handle_handoff_arm(peer_id, entry_edge);
                }
                FlowEvent::FlowHandoffDisarm { peer_id } => {
                    crate::flow::handoff::handle_handoff_disarm(peer_id);
                }
                FlowEvent::FlowReceptionAck { peer_id } => {
                    crate::flow::handoff::handle_reception_ack(peer_id);
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

                    let mode = FLOW_MANAGER.flow_mouse_mode.read().unwrap().clone();
                    if mode == "hardware" {
                        crate::flow::switching::trigger_hidpp_channel_switch(0);
                    }

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

pub async fn send_event_to_peer(peer_name: &str, event: &FlowEvent) -> anyhow::Result<()> {
    let serialized = serde_json::to_string(event)?;
    let s = {
        let conns = ACTIVE_CONNECTIONS.read().unwrap();
        conns.get(peer_name).cloned()
    };
    if let Some(s) = s {
        let mut guard = s.lock().await;
        use tokio::io::AsyncWriteExt;
        guard.write_all(format!("{}\n", serialized).as_bytes()).await?;
        guard.flush().await?;
    }
    Ok(())
}

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
            FlowEvent::FlowHandoffRequest {
                peer_id: "peer_req".to_string(),
                entry_edge: crate::flow::edge::EdgeEvent::Left,
                cursor_pos: (100, 200),
            },
            FlowEvent::FlowHandoffAck {
                peer_id: "peer_ack".to_string(),
            },
            FlowEvent::FlowHandoffArm {
                peer_id: "peer_arm".to_string(),
                entry_edge: crate::flow::edge::EdgeEvent::Right,
            },
            FlowEvent::FlowHandoffDisarm {
                peer_id: "peer_disarm".to_string(),
            },
            FlowEvent::FlowReceptionAck {
                peer_id: "peer_rec".to_string(),
            },
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
                (
                    FlowEvent::FlowHandoffRequest { peer_id: p1, entry_edge: e1, cursor_pos: c1 },
                    FlowEvent::FlowHandoffRequest { peer_id: p2, entry_edge: e2, cursor_pos: c2 },
                ) => {
                    assert_eq!(p1, p2);
                    assert_eq!(e1, e2);
                    assert_eq!(c1, c2);
                }
                (
                    FlowEvent::FlowHandoffAck { peer_id: p1 },
                    FlowEvent::FlowHandoffAck { peer_id: p2 },
                ) => {
                    assert_eq!(p1, p2);
                }
                (
                    FlowEvent::FlowHandoffArm { peer_id: p1, entry_edge: e1 },
                    FlowEvent::FlowHandoffArm { peer_id: p2, entry_edge: e2 },
                ) => {
                    assert_eq!(p1, p2);
                    assert_eq!(e1, e2);
                }
                (
                    FlowEvent::FlowHandoffDisarm { peer_id: p1 },
                    FlowEvent::FlowHandoffDisarm { peer_id: p2 },
                ) => {
                    assert_eq!(p1, p2);
                }
                (
                    FlowEvent::FlowReceptionAck { peer_id: p1 },
                    FlowEvent::FlowReceptionAck { peer_id: p2 },
                ) => {
                    assert_eq!(p1, p2);
                }
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

        let client_hash = expected_hash.clone();
        let verification_input = format!("{}{}", salt, pin);
        let verified_hash = format!("{:x}", sha2::Sha256::digest(verification_input.as_bytes()));
        assert_eq!(client_hash, verified_hash);
    }
}
