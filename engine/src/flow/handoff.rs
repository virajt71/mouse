use std::sync::atomic::Ordering;
use std::sync::{Mutex, LazyLock};
use std::time::{Duration, Instant};
use crate::flow::edge::{EdgeEvent, EdgeZoneEvent};
use crate::flow::topology::{resolve_peer, FlowPeer};
use crate::flow::network::{send_event_to_peer, FlowEvent};
use crate::flow::FLOW_MANAGER;
use crate::lock_ext::MutexExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowState {
    Idle,
    Armed {
        edge: EdgeEvent,
        armed_at: Instant,
    },
    HandoffPending {
        edge: EdgeEvent,
        fired_at: Instant,
    },
    Forwarding,
    CooldownRecovery {
        until: Instant,
        attempt_count: u32,
    },
}

#[derive(Debug, Clone)]
pub struct HandoffContext {
    pub state: FlowState,
    pub target_peer: Option<FlowPeer>,
    pub consecutive_failures: u32,
    pub last_switch_completed: Option<Instant>,
    pub expected_handoff: Option<(String, EdgeEvent)>,
}

impl Default for HandoffContext {
    fn default() -> Self {
        HandoffContext {
            state: FlowState::Idle,
            target_peer: None,
            consecutive_failures: 0,
            last_switch_completed: None,
            expected_handoff: None,
        }
    }
}

pub static HANDOFF_CTX: LazyLock<Mutex<HandoffContext>> =
    LazyLock::new(|| Mutex::new(HandoffContext::default()));

pub fn get_state() -> FlowState {
    HANDOFF_CTX.lock().unwrap().state
}

pub fn get_expected_handoff() -> Option<(String, EdgeEvent)> {
    HANDOFF_CTX.lock().unwrap().expected_handoff.clone()
}

pub fn handle_handoff_arm(sender_peer_id: String, entry_edge: EdgeEvent) {
    log::info!("[Handoff] Arm received from '{}', preparing resources...", sender_peer_id);
    let mut ctx = HANDOFF_CTX.lock().unwrap();
    ctx.expected_handoff = Some((sender_peer_id, entry_edge));
}

pub fn handle_handoff_disarm(sender_peer_id: String) {
    log::info!("[Handoff] Disarm received from '{}', clearing prep...", sender_peer_id);
    let mut ctx = HANDOFF_CTX.lock().unwrap();
    if let Some((ref expected_peer, _)) = ctx.expected_handoff {
        if expected_peer == &sender_peer_id {
            ctx.expected_handoff = None;
        }
    }
}

pub fn handle_reception_ack(sender_peer_id: String) {
    log::info!("[Handoff] Received ReceptionAck from peer '{}'. Channel switch confirmed.", sender_peer_id);
    let mut ctx = HANDOFF_CTX.lock().unwrap();
    ctx.last_switch_completed = Some(Instant::now());
}

fn show_notification(title: &str, body: &str) {
    log::warn!("Notification: {} - {}", title, body);
    let _ = std::process::Command::new("notify-send")
        .arg(title)
        .arg(body)
        .arg("-t")
        .arg("5000")
        .spawn();
}

pub async fn handle_warning_line(edge: EdgeEvent) {
    let to_send = {
        let mut ctx = HANDOFF_CTX.lock().unwrap();
        
        // Cooldown check
        if let Some(last) = ctx.last_switch_completed {
            let cooldown = FLOW_MANAGER.flow_cooldown_ms.load(Ordering::Relaxed);
            if last.elapsed() < Duration::from_millis(cooldown) {
                log::debug!("[Handoff] Cooldown active, ignoring warning line");
                return;
            }
        }
        
        // Only transition to Armed from Idle or from CooldownRecovery if expired
        match ctx.state {
            FlowState::Idle => {},
            FlowState::CooldownRecovery { until, .. } => {
                if Instant::now() < until {
                    log::debug!("[Handoff] In active cooldown recovery, ignoring warning line");
                    return;
                }
                ctx.state = FlowState::Idle;
            }
            _ => {
                log::debug!("[Handoff] State is already {:?}, ignoring warning line", ctx.state);
                return;
            }
        }
        
        if let Some(peer) = resolve_peer(edge) {
            log::info!("[Handoff] Warning line crossed. Transitioning Idle -> Armed for peer '{}'", peer.peer_id);
            ctx.state = FlowState::Armed {
                edge,
                armed_at: Instant::now(),
            };
            ctx.target_peer = Some(peer.clone());
            
            let local_name = FLOW_MANAGER.flow_local_name.read().unwrap().clone();
            let event = FlowEvent::FlowHandoffArm {
                peer_id: local_name,
                entry_edge: match edge {
                    EdgeEvent::Left => EdgeEvent::Right,
                    EdgeEvent::Right => EdgeEvent::Left,
                    EdgeEvent::Top => EdgeEvent::Bottom,
                    EdgeEvent::Bottom => EdgeEvent::Top,
                },
            };
            Some((peer, event))
        } else {
            None
        }
    };
    
    if let Some((peer, event)) = to_send {
        if let Err(e) = send_event_to_peer(&peer.peer_id, &event).await {
            log::error!("[Handoff] Failed to send FlowHandoffArm to '{}': {}", peer.peer_id, e);
            let mut ctx = HANDOFF_CTX.lock().unwrap();
            ctx.state = FlowState::Idle;
            ctx.target_peer = None;
        } else {
            let arm_timeout = FLOW_MANAGER.flow_arm_timeout_ms.load(Ordering::Relaxed);
            let check_at = Instant::now();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(arm_timeout)).await;
                let mut ctx = HANDOFF_CTX.lock().unwrap();
                if let FlowState::Armed { armed_at, .. } = ctx.state {
                    if armed_at == check_at {
                        log::info!("[Handoff] Armed state timed out, disarming...");
                        ctx.state = FlowState::Idle;
                        ctx.target_peer = None;
                        let local_name = FLOW_MANAGER.flow_local_name.read().unwrap().clone();
                        let disarm_event = FlowEvent::FlowHandoffDisarm { peer_id: local_name };
                        let peer_id = peer.peer_id.clone();
                        tokio::spawn(async move {
                            let _ = send_event_to_peer(&peer_id, &disarm_event).await;
                        });
                    }
                }
            });
        }
    }
}

pub async fn cancel_arm() {
    let to_send = {
        let mut ctx = HANDOFF_CTX.lock().unwrap();
        if let FlowState::Armed { .. } = ctx.state {
            log::info!("[Handoff] Retreat detected. Transitioning Armed -> Idle.");
            ctx.state = FlowState::Idle;
            let peer = ctx.target_peer.take();
            let local_name = FLOW_MANAGER.flow_local_name.read().unwrap().clone();
            let disarm_event = FlowEvent::FlowHandoffDisarm { peer_id: local_name };
            peer.map(|p| (p, disarm_event))
        } else {
            None
        }
    };
    
    if let Some((peer, event)) = to_send {
        if let Err(e) = send_event_to_peer(&peer.peer_id, &event).await {
            log::warn!("[Handoff] Failed to send FlowHandoffDisarm to '{}': {}", peer.peer_id, e);
        }
    }
}

pub async fn handle_fire_line(edge: EdgeEvent) {
    let to_send = {
        let mut ctx = HANDOFF_CTX.lock().unwrap();
        
        if ctx.state == FlowState::Idle {
            if let Some(peer) = resolve_peer(edge) {
                ctx.target_peer = Some(peer);
                ctx.state = FlowState::Armed {
                    edge,
                    armed_at: Instant::now(),
                };
            }
        }
        
        if let FlowState::Armed { .. } = ctx.state {
            if let Some(peer) = ctx.target_peer.clone() {
                log::info!("[Handoff] Screen edge crossed. Transitioning Armed -> HandoffPending. Sending request to '{}'", peer.peer_id);
                ctx.state = FlowState::HandoffPending {
                    edge,
                    fired_at: Instant::now(),
                };
                
                let local_name = FLOW_MANAGER.flow_local_name.read().unwrap().clone();
                let cx = *FLOW_MANAGER.virtual_x.lock_safe();
                let cy = *FLOW_MANAGER.virtual_y.lock_safe();
                
                let entry_edge = match edge {
                    EdgeEvent::Left => EdgeEvent::Right,
                    EdgeEvent::Right => EdgeEvent::Left,
                    EdgeEvent::Top => EdgeEvent::Bottom,
                    EdgeEvent::Bottom => EdgeEvent::Top,
                };
                
                let request = FlowEvent::FlowHandoffRequest {
                    peer_id: local_name,
                    entry_edge,
                    cursor_pos: (cx, cy),
                };
                
                Some((peer, request))
            } else {
                None
            }
        } else {
            None
        }
    };
    
    if let Some((peer, request)) = to_send {
        if let Err(e) = send_event_to_peer(&peer.peer_id, &request).await {
            log::error!("[Handoff] Failed to send handoff request: {}. Aborting.", e);
            let mut ctx = HANDOFF_CTX.lock().unwrap();
            ctx.state = FlowState::Idle;
            ctx.target_peer = None;
        } else {
            let timeout_ms = FLOW_MANAGER.flow_ack_timeout_ms.load(Ordering::Relaxed);
            let peer_id_clone = peer.peer_id.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
                check_timeout(&peer_id_clone).await;
            });
        }
    }
}

pub fn handle_handoff_request(sender_peer_id: String, entry_edge: EdgeEvent, cursor_pos: (i32, i32), client_name: &str) {
    log::info!(
        "[Handoff] Handling handoff request from '{}' (client_name='{}'). Entering at {:?}",
        sender_peer_id, client_name, entry_edge
    );

    let local_name = FLOW_MANAGER.flow_local_name.read().unwrap().clone();
    let ack = FlowEvent::FlowHandoffAck { peer_id: local_name };
    let client_name_clone = client_name.to_string();
    tokio::spawn(async move {
        if let Err(e) = send_event_to_peer(&client_name_clone, &ack).await {
            log::error!("[Handoff] Failed to send handoff ACK: {}", e);
        }
    });

    {
        let mut ctx = HANDOFF_CTX.lock().unwrap();
        ctx.state = FlowState::Idle;
        ctx.expected_handoff = None;
    }

    FLOW_MANAGER.set_active_peer(None);

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

    let client_name_for_rec = client_name.to_string();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let local_name = FLOW_MANAGER.flow_local_name.read().unwrap().clone();
        let rec_ack = FlowEvent::FlowReceptionAck { peer_id: local_name };
        let _ = send_event_to_peer(&client_name_for_rec, &rec_ack).await;
    });
}

pub fn handle_handoff_ack(sender_peer_id: String) {
    let mut ctx = HANDOFF_CTX.lock().unwrap();
    if let FlowState::HandoffPending { .. } = ctx.state {
        if let Some(peer) = ctx.target_peer.clone() {
            if peer.peer_id == sender_peer_id {
                log::info!("[Handoff] Received ACK from target '{}'. Transitioning HandoffPending -> Forwarding.", sender_peer_id);
                
                ctx.state = FlowState::Forwarding;
                FLOW_MANAGER.set_active_peer(Some(peer.peer_id.clone()));
                ctx.consecutive_failures = 0;

                let target_channel = peer.channel;
                log::info!("[Handoff] Triggering local HID++ switch to channel {}", target_channel);
                
                if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock_safe() {
                    let mut clients = inner.hid_clients.lock_safe();
                    for client in clients.iter_mut() {
                        if client.change_host_idx.is_some() {
                            log::info!("[Handoff] Performing change_host on device '{}'", client.device_name);
                            if let Err(e) = client.switch_host_channel(target_channel) {
                                log::error!("[Handoff] Failed to switch channel on device '{}': {}", client.device_name, e);
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
}

async fn check_timeout(peer_id: &str) {
    let (consecutive_failures, target_peer) = {
        let mut ctx = HANDOFF_CTX.lock().unwrap();
        if let FlowState::HandoffPending { .. } = ctx.state {
            if let Some(ref peer) = ctx.target_peer {
                if peer.peer_id == peer_id {
                    ctx.consecutive_failures += 1;
                    let failures = ctx.consecutive_failures;
                    
                    let backoff_ms = {
                        let base_ms = FLOW_MANAGER.flow_cooldown_ms.load(Ordering::Relaxed);
                        let max_ms = FLOW_MANAGER.flow_max_backoff_ms.load(Ordering::Relaxed);
                        let computed = base_ms * (2u64.pow(failures.min(5)));
                        computed.min(max_ms)
                    };
                    
                    log::warn!(
                        "[Handoff] ACK timeout for '{}' (attempt {}). Entering CooldownRecovery for {}ms.",
                        peer_id, failures, backoff_ms
                    );
                    
                    ctx.state = FlowState::CooldownRecovery {
                        until: Instant::now() + Duration::from_millis(backoff_ms),
                        attempt_count: failures,
                    };
                    ctx.last_switch_completed = Some(Instant::now());
                    
                    let target_peer = ctx.target_peer.clone();
                    ctx.target_peer = None;
                    
                    (failures, target_peer)
                } else {
                    return;
                }
            } else {
                return;
            }
        } else {
            return;
        }
    };
    
    // Recovery actions: bounce cursor back, show hint to user
    let sw = *FLOW_MANAGER.screen_width.read().unwrap();
    let sh = *FLOW_MANAGER.screen_height.read().unwrap();
    let mut vx = FLOW_MANAGER.virtual_x.lock_safe();
    let mut vy = FLOW_MANAGER.virtual_y.lock_safe();
    
    if let Some(peer) = target_peer {
        match peer.edge_relation {
            EdgeEvent::Left => *vx = 20,
            EdgeEvent::Right => *vx = sw - 20,
            EdgeEvent::Top => *vy = 20,
            EdgeEvent::Bottom => *vy = sh - 20,
        }
        
        // Sync X11 cursor position to reflect bounced coordinates
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
                *vx as i16,
                *vy as i16,
            );
            let _ = conn.flush();
        }
    }

    show_notification(
        "Mouser Flow",
        &format!(
            "Handoff to {} timed out! (Attempt {}). If your mouse channel switched, press Easy-Switch to return to Channel 1.",
            peer_id, consecutive_failures
        )
    );
}

pub async fn run_handoff_loop() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    {
        let mut guard = crate::flow::edge::EDGE_SENDER.lock().unwrap();
        *guard = Some(tx);
    }

    log::info!("[Handoff] Loop started, waiting for EdgeZoneEvents...");

    while let Some(zone_ev) = rx.recv().await {
        match zone_ev {
            EdgeZoneEvent::EnteredPrepZone(edge) => {
                handle_warning_line(edge).await;
            }
            EdgeZoneEvent::ReturnedToSafeZone(_edge) => {
                cancel_arm().await;
            }
            EdgeZoneEvent::HitScreenEdge(edge) => {
                handle_fire_line(edge).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let ctx = HANDOFF_CTX.lock().unwrap();
        assert_eq!(ctx.state, FlowState::Idle);
        assert!(ctx.target_peer.is_none());
        assert_eq!(ctx.consecutive_failures, 0);
        assert!(ctx.expected_handoff.is_none());
    }

    #[test]
    fn test_expected_handoff_flow() {
        // Arm
        handle_handoff_arm("peer_1".to_string(), EdgeEvent::Left);
        assert_eq!(get_expected_handoff(), Some(("peer_1".to_string(), EdgeEvent::Left)));

        // Disarm
        handle_handoff_disarm("peer_1".to_string());
        assert!(get_expected_handoff().is_none());
    }

    #[test]
    fn test_consecutive_failures_backoff() {
        // Reset state inside HANDOFF_CTX
        {
            let mut ctx = HANDOFF_CTX.lock().unwrap();
            ctx.state = FlowState::Idle;
            ctx.consecutive_failures = 0;
            ctx.target_peer = None;
            ctx.last_switch_completed = None;
        }

        // Configure a mock peer in FLOW_MANAGER
        let peer = crate::config::FlowPeer {
            name: "test_peer".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 5000,
            layout_x: -1,
            layout_y: 0,
            paired: true,
            fingerprint: "".to_string(),
            auto_reconnect: false,
            channel_index: 2,
        };
        *FLOW_MANAGER.flow_peers.write().unwrap() = vec![peer];
        FLOW_MANAGER.flow_warning_line_distance.store(80, Ordering::Relaxed);
        FLOW_MANAGER.flow_cooldown_ms.store(300, Ordering::Relaxed);
        FLOW_MANAGER.flow_max_backoff_ms.store(10000, Ordering::Relaxed);

        // Trigger warning line Left
        let rx = crate::TOKIO_RUNTIME.block_on(async {
            handle_warning_line(EdgeEvent::Left).await;
            get_state()
        });
        
        assert!(matches!(rx, FlowState::Armed { edge: EdgeEvent::Left, .. }));
        
        // Trigger cancel
        crate::TOKIO_RUNTIME.block_on(async {
            cancel_arm().await;
        });
        assert_eq!(get_state(), FlowState::Idle);
    }
}
