use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use crate::flow::edge::EdgeEvent;
use crate::flow::topology::{resolve_peer, FlowPeer};
use crate::flow::network::{send_event_to_peer, FlowEvent};
use crate::flow::FLOW_MANAGER;
use crate::lock_ext::MutexExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowState {
    Active,
    HandoffPending,
    Inactive,
}

pub static FLOW_STATE: std::sync::LazyLock<Mutex<FlowState>> =
    std::sync::LazyLock::new(|| Mutex::new(FlowState::Active));

pub static HANDOFF_PENDING_PEER: std::sync::LazyLock<Mutex<Option<FlowPeer>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

pub static HANDOFF_START_TIME: std::sync::LazyLock<Mutex<Option<Instant>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

pub fn get_state() -> FlowState {
    *FLOW_STATE.lock().unwrap()
}

pub fn handle_edge_event(edge: EdgeEvent) {
    let mut state_guard = FLOW_STATE.lock().unwrap();
    if *state_guard != FlowState::Active {
        log::debug!("[Handoff] Edge event {:?} ignored because state is {:?}", edge, *state_guard);
        return;
    }

    log::info!("[Handoff] Edge event detected: {:?}", edge);

    // Resolve topology
    if let Some(peer) = resolve_peer(edge) {
        *state_guard = FlowState::HandoffPending;
        
        let local_name = FLOW_MANAGER.flow_local_name.read().unwrap().clone();
        
        // Read current cursor position from virtual_x and virtual_y
        let cx = *FLOW_MANAGER.virtual_x.lock_safe();
        let cy = *FLOW_MANAGER.virtual_y.lock_safe();

        // The entry edge on the receiver side is the opposite of the sender's exit edge:
        // Left exit -> Right entry
        // Right exit -> Left entry
        // Top exit -> Bottom entry
        // Bottom exit -> Top entry
        let entry_edge = match edge {
            EdgeEvent::Left => EdgeEvent::Right,
            EdgeEvent::Right => EdgeEvent::Left,
            EdgeEvent::Top => EdgeEvent::Bottom,
            EdgeEvent::Bottom => EdgeEvent::Top,
        };

        log::info!("[Handoff] Transitioning Active -> HandoffPending. Sending request to '{}'", peer.peer_id);
        
        // Store pending details
        *HANDOFF_PENDING_PEER.lock().unwrap() = Some(peer.clone());
        *HANDOFF_START_TIME.lock().unwrap() = Some(Instant::now());

        let req = FlowEvent::FlowHandoffRequest {
            peer_id: local_name,
            entry_edge,
            cursor_pos: (cx, cy),
        };

        if let Err(e) = send_event_to_peer(&peer.peer_id, &req) {
            log::error!("[Handoff] Failed to send handoff request: {}. Aborting.", e);
            *state_guard = FlowState::Active;
            *HANDOFF_PENDING_PEER.lock().unwrap() = None;
            *HANDOFF_START_TIME.lock().unwrap() = None;
        } else {
            // Spawn timeout checker thread
            let timeout_ms = FLOW_MANAGER.flow_handoff_timeout_ms.load(Ordering::Relaxed);
            let peer_id_clone = peer.peer_id.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(timeout_ms));
                check_timeout(&peer_id_clone);
            });
        }
    } else {
        log::debug!("[Handoff] No peer configured for edge {:?}", edge);
    }
}

pub fn handle_handoff_request(sender_peer_id: String, entry_edge: EdgeEvent, cursor_pos: (i32, i32), client_name: &str) {
    log::info!(
        "[Handoff] Handling handoff request from '{}' (client_name='{}'). Entering at {:?}",
        sender_peer_id, client_name, entry_edge
    );

    // Send ACK first
    let local_name = FLOW_MANAGER.flow_local_name.read().unwrap().clone();
    let ack = FlowEvent::FlowHandoffAck { peer_id: local_name };
    if let Err(e) = send_event_to_peer(client_name, &ack) {
        log::error!("[Handoff] Failed to send handoff ACK: {}", e);
        return;
    }

    // Update state to Active (we are now receiving the mouse)
    {
        let mut state_guard = FLOW_STATE.lock().unwrap();
        *state_guard = FlowState::Active;
    }

    // Set FLOW_MANAGER active peer to None (we are controlling local cursor now)
    FLOW_MANAGER.set_active_peer(None);

    // Calculate target cursor position
    let sw = *FLOW_MANAGER.screen_width.read().unwrap();
    let sh = *FLOW_MANAGER.screen_height.read().unwrap();

    let (target_x, target_y) = match entry_edge {
        EdgeEvent::Left => (50, cursor_pos.1),
        EdgeEvent::Right => (sw - 50, cursor_pos.1),
        EdgeEvent::Top => (cursor_pos.0, 50),
        EdgeEvent::Bottom => (cursor_pos.0, sh - 50),
    };

    let target_x = target_x.clamp(0, sw);
    let target_y = target_y.clamp(0, sh);

    *FLOW_MANAGER.virtual_x.lock_safe() = target_x;
    *FLOW_MANAGER.virtual_y.lock_safe() = target_y;

    log::info!("[Handoff] Seeding/injecting cursor position to ({}, {})", target_x, target_y);

    // Warp pointer under X11
    if let Ok((conn, screen_num)) = x11rb::rust_connection::RustConnection::connect(None) {
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

pub fn handle_handoff_ack(sender_peer_id: String) {
    let mut state_guard = FLOW_STATE.lock().unwrap();
    if *state_guard != FlowState::HandoffPending {
        log::warn!("[Handoff] Received ACK from '{}' but state is {:?}", sender_peer_id, *state_guard);
        return;
    }

    let pending_peer = HANDOFF_PENDING_PEER.lock().unwrap().clone();
    if let Some(peer) = pending_peer {
        if peer.peer_id == sender_peer_id {
            log::info!("[Handoff] Received ACK from target '{}'. Transitioning HandoffPending -> Inactive.", sender_peer_id);
            
            *state_guard = FlowState::Inactive;
            
            // Set active peer name so we know who is controlling
            FLOW_MANAGER.set_active_peer(Some(peer.peer_id.clone()));

            // Clear pending details
            *HANDOFF_PENDING_PEER.lock().unwrap() = None;
            *HANDOFF_START_TIME.lock().unwrap() = None;

            // Trigger physical HID++ channel switch to peer's channel
            let target_channel = peer.channel;
            log::info!("[Handoff] Triggering local HID++ switch to channel {}", target_channel);
            
            // Call change_host on all connected mice that support CHANGE_HOST
            if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock_safe() {
                let clients = inner.hid_clients.lock_safe();
                for client in clients.iter() {
                    if client.change_host_idx.is_some() {
                        if let Some(ref dev) = client.device {
                            log::info!("[Handoff] Performing change_host on device '{}'", client.device_name);
                            if let Err(e) = crate::flow::channel_switch::change_host(dev, target_channel) {
                                log::error!("[Handoff] Failed to switch channel on device '{}': {}", client.device_name, e);
                            }
                        }
                    }
                }
            }
        } else {
            log::warn!(
                "[Handoff] Received ACK from '{}' but expected ACK from pending peer '{}'",
                sender_peer_id, peer.peer_id
            );
        }
    }
}

fn check_timeout(peer_id: &str) {
    let mut state_guard = FLOW_STATE.lock().unwrap();
    if *state_guard == FlowState::HandoffPending {
        let pending = HANDOFF_PENDING_PEER.lock().unwrap().clone();
        if let Some(peer) = pending {
            if peer.peer_id == peer_id {
                log::warn!(
                    "[Handoff] Handoff request to '{}' timed out! Aborting and remaining Active.",
                    peer_id
                );
                *state_guard = FlowState::Active;
                *HANDOFF_PENDING_PEER.lock().unwrap() = None;
                *HANDOFF_START_TIME.lock().unwrap() = None;

                // Warp virtual cursor back slightly to prevent immediate re-triggering
                let sw = *FLOW_MANAGER.screen_width.read().unwrap();
                let sh = *FLOW_MANAGER.screen_height.read().unwrap();
                let mut vx = FLOW_MANAGER.virtual_x.lock_safe();
                let mut vy = FLOW_MANAGER.virtual_y.lock_safe();
                match peer.edge_relation {
                    EdgeEvent::Left => *vx = 20,
                    EdgeEvent::Right => *vx = sw - 20,
                    EdgeEvent::Top => *vy = 20,
                    EdgeEvent::Bottom => *vy = sh - 20,
                }
            }
        }
    }
}

pub fn run_handoff_loop() {
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let mut guard = crate::flow::edge::EDGE_SENDER.lock().unwrap();
        *guard = Some(tx);
    }

    log::info!("[Handoff] Loop started, waiting for EdgeEvents...");

    while let Ok(edge_ev) = rx.recv() {
        handle_edge_event(edge_ev);
    }
}
