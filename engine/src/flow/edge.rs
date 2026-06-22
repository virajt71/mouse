use std::sync::atomic::Ordering;
use tokio::sync::mpsc::UnboundedSender;
use std::sync::Arc;
use std::time::Duration;
use crate::flow::FLOW_MANAGER;
use crate::lock_ext::MutexExt;
use x11rb::connection::Connection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EdgeEvent {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EdgeZoneEvent {
    EnteredPrepZone(EdgeEvent),
    ReturnedToSafeZone(EdgeEvent),
    HitScreenEdge(EdgeEvent),
}

pub static EDGE_SENDER: std::sync::Mutex<Option<UnboundedSender<EdgeZoneEvent>>> = std::sync::Mutex::new(None);

pub fn emit_edge_event(event: EdgeZoneEvent) {
    if let Ok(guard) = EDGE_SENDER.lock() {
        if let Some(ref tx) = *guard {
            let _ = tx.send(event);
        }
    }
}

pub fn is_ctrl_held() -> bool {
    if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock_safe() {
        inner.modifier_state.ctrl.load(Ordering::Relaxed)
    } else {
        false
    }
}

pub async fn run_x11_edge_polling(engine_inner: Arc<crate::engine::inner::EngineInner>) {
    loop {
        if !engine_inner.running.load(Ordering::Acquire) {
            break;
        }

        let enabled = FLOW_MANAGER.flow_enabled.load(Ordering::Relaxed);
        if !enabled {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            continue;
        }

        let x11_conn = x11rb::rust_connection::RustConnection::connect(None);
        match x11_conn {
            Ok((conn, screen_num)) => {
                log::info!("[Flow Edge] X11 QueryPointer edge detection running...");
                let screen = &conn.setup().roots[screen_num];
                let root = screen.root;
                let mut armed_edge: Option<EdgeEvent> = None;

                loop {
                    if !engine_inner.running.load(Ordering::Acquire) {
                        return;
                    }

                    let inner_enabled = FLOW_MANAGER.flow_enabled.load(Ordering::Relaxed);
                    if !inner_enabled {
                        break;
                    }

                    let is_active = FLOW_MANAGER.get_active_peer_name().is_none();
                    let current_sleep;

                    if is_active {
                        use x11rb::protocol::xproto::ConnectionExt;
                        match conn.query_pointer(root) {
                            Ok(cookie) => {
                                match cookie.reply() {
                                    Ok(reply) => {
                                        let rx = reply.root_x as i32;
                                        let ry = reply.root_y as i32;
                                        let sw = screen.width_in_pixels as i32;
                                        let sh = screen.height_in_pixels as i32;

                                        *FLOW_MANAGER.screen_width.write().unwrap() = sw;
                                        *FLOW_MANAGER.screen_height.write().unwrap() = sh;

                                        let edge_threshold = FLOW_MANAGER.flow_edge_threshold.load(Ordering::Relaxed);
                                        let warning_dist = FLOW_MANAGER.flow_warning_line_distance.load(Ordering::Relaxed);
                                        let hysteresis = 40;

                                        let dist_left = rx;
                                        let dist_right = sw - rx;
                                        let dist_top = ry;
                                        let dist_bottom = sh - ry;

                                        let (closest_edge, dist) = if dist_left <= dist_right && dist_left <= dist_top && dist_left <= dist_bottom {
                                            (EdgeEvent::Left, dist_left)
                                        } else if dist_right <= dist_left && dist_right <= dist_top && dist_right <= dist_bottom {
                                            (EdgeEvent::Right, dist_right)
                                        } else if dist_top <= dist_left && dist_top <= dist_right && dist_top <= dist_bottom {
                                            (EdgeEvent::Top, dist_top)
                                        } else {
                                            (EdgeEvent::Bottom, dist_bottom)
                                        };

                                        // Dynamic polling sleep calculation to reduce CPU usage
                                        current_sleep = if dist < 50 {
                                            Duration::from_millis(30)
                                        } else if dist < 150 {
                                            Duration::from_millis(100)
                                        } else {
                                            Duration::from_millis(250)
                                        };

                                        let hold_key = FLOW_MANAGER.flow_hold_key.read().unwrap().clone();
                                        let ctrl_only = FLOW_MANAGER.flow_hold_ctrl_only.load(Ordering::Relaxed);
                                        
                                        let mut satisfied = super::switching::is_hold_key_satisfied(&hold_key);
                                        if ctrl_only {
                                            satisfied = satisfied && is_ctrl_held();
                                        }

                                        if satisfied {
                                            let mode = FLOW_MANAGER.flow_mouse_mode.read().unwrap().clone();
                                            if mode == "hardware" {
                                                // Handle disarming first (retreat from prep zone back to safe zone)
                                                if let Some(armed) = armed_edge {
                                                    if armed != closest_edge || dist > warning_dist + hysteresis {
                                                        emit_edge_event(EdgeZoneEvent::ReturnedToSafeZone(armed));
                                                        armed_edge = None;
                                                    }
                                                }

                                                // Handle arming / fire
                                                if dist <= edge_threshold {
                                                    // Fire screen edge crossing!
                                                    // If we skipped the prep zone (fast sweep), make sure we arm first
                                                    if armed_edge.is_none() {
                                                        emit_edge_event(EdgeZoneEvent::EnteredPrepZone(closest_edge));
                                                        armed_edge = Some(closest_edge);
                                                    }
                                                    emit_edge_event(EdgeZoneEvent::HitScreenEdge(closest_edge));
                                                } else if dist <= warning_dist {
                                                    // Arm prep zone!
                                                    if armed_edge.is_none() {
                                                        emit_edge_event(EdgeZoneEvent::EnteredPrepZone(closest_edge));
                                                        armed_edge = Some(closest_edge);
                                                    }
                                                }
                                            } else {
                                                // Software mode (legacy/direct transition at screen edge)
                                                if dist <= edge_threshold {
                                                    if let Some(peer) = super::topology::resolve_peer(closest_edge) {
                                                        log::info!("[Flow Edge] Software mode edge crossed: transitioning to peer '{}'", peer.peer_id);
                                                        FLOW_MANAGER.set_active_peer(Some(peer.peer_id.clone()));
                                                        
                                                        let rx_target = match closest_edge {
                                                            EdgeEvent::Left => 100,
                                                            EdgeEvent::Right => sw - 100,
                                                            _ => rx,
                                                        };
                                                        let ry_target = match closest_edge {
                                                            EdgeEvent::Top => 100,
                                                            EdgeEvent::Bottom => sh - 100,
                                                            _ => ry,
                                                        };
                                                        let _ = conn.warp_pointer(
                                                            x11rb::NONE,
                                                            root,
                                                            0,
                                                            0,
                                                            0,
                                                            0,
                                                            rx_target as i16,
                                                            ry_target as i16,
                                                        );
                                                        let _ = conn.flush();
                                                    }
                                                }
                                            }
                                        } else {
                                            // If hold key is released and we were armed, cancel arming
                                            if let Some(armed) = armed_edge {
                                                emit_edge_event(EdgeZoneEvent::ReturnedToSafeZone(armed));
                                                armed_edge = None;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("[Flow Edge] X11 query reply error: {}. Reconnecting...", e);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("[Flow Edge] X11 query pointer error: {}. Reconnecting...", e);
                                break;
                            }
                        }
                    } else {
                        // When inactive, poll slower
                        current_sleep = Duration::from_millis(500);
                    }

                    tokio::time::sleep(current_sleep).await;
                }
            }
            Err(e) => {
                log::debug!("[Flow Edge] X11 connection failed: {}. Retrying in 5s...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
