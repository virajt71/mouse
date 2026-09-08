use crate::flow::FLOW_MANAGER;
use crate::lock_ext::MutexExt;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;
use x11rb::connection::Connection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EdgeEvent {
    Left,
    Right,
    Top,
    Bottom,
}

pub static EDGE_SENDER: std::sync::Mutex<Option<Sender<EdgeEvent>>> = std::sync::Mutex::new(None);

pub fn emit_edge_event(event: EdgeEvent) {
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

pub fn run_x11_edge_polling(engine_inner: Arc<crate::engine::inner::EngineInner>) {
    loop {
        if !engine_inner.running.load(Ordering::Acquire) {
            break;
        }

        let enabled = FLOW_MANAGER.flow_enabled.load(Ordering::Relaxed);
        if !enabled {
            std::thread::sleep(Duration::from_millis(1000));
            continue;
        }

        let x11_conn = x11rb::rust_connection::RustConnection::connect(None);
        match x11_conn {
            Ok((conn, screen_num)) => {
                log::info!("[Flow Edge] X11 QueryPointer edge detection running...");
                let screen = &conn.setup().roots[screen_num];
                let root = screen.root;

                loop {
                    if !engine_inner.running.load(Ordering::Acquire) {
                        return;
                    }

                    let inner_enabled = FLOW_MANAGER.flow_enabled.load(Ordering::Relaxed);
                    if !inner_enabled {
                        break;
                    }

                    // Check if we are currently active (local input is active)
                    let is_active = FLOW_MANAGER.get_active_peer_name().is_none();

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

                                        // Update dimensions
                                        FLOW_MANAGER.screen_width.store(sw, Ordering::Relaxed);
                                        FLOW_MANAGER.screen_height.store(sh, Ordering::Relaxed);

                                        let threshold = FLOW_MANAGER
                                            .flow_edge_threshold
                                            .load(Ordering::Relaxed);
                                        let mut edge = None;

                                        if rx <= threshold {
                                            edge = Some(EdgeEvent::Left);
                                        } else if rx >= sw - threshold - 1 {
                                            edge = Some(EdgeEvent::Right);
                                        } else if ry <= threshold {
                                            edge = Some(EdgeEvent::Top);
                                        } else if ry >= sh - threshold - 1 {
                                            edge = Some(EdgeEvent::Bottom);
                                        }

                                        if let Some(ev) = edge {
                                            let hold_key =
                                                FLOW_MANAGER.flow_hold_key.read().unwrap().clone();
                                            let ctrl_only = FLOW_MANAGER
                                                .flow_hold_ctrl_only
                                                .load(Ordering::Relaxed);

                                            let mut satisfied =
                                                super::switching::is_hold_key_satisfied(&hold_key);
                                            if ctrl_only {
                                                satisfied = satisfied && is_ctrl_held();
                                            }

                                            if satisfied {
                                                let mode = FLOW_MANAGER
                                                    .flow_mouse_mode
                                                    .read()
                                                    .unwrap()
                                                    .clone();
                                                if mode == "hardware" {
                                                    emit_edge_event(ev);
                                                } else {
                                                    // Software mode legacy/existing transition logic
                                                    if let Some(peer) =
                                                        super::topology::resolve_peer(ev)
                                                    {
                                                        log::info!("[Flow Edge] Software mode edge crossed: transitioning to peer '{}'", peer.peer_id);
                                                        FLOW_MANAGER.set_active_peer(Some(
                                                            peer.peer_id.clone(),
                                                        ));

                                                        // Reset virtual coords/pointer slightly away to prevent loop bouncing
                                                        let rx_target = match ev {
                                                            EdgeEvent::Left => 100,
                                                            EdgeEvent::Right => sw - 100,
                                                            _ => rx,
                                                        };
                                                        let ry_target = match ev {
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
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("[Flow Edge] X11 query reply error: {}. Reconnecting...", e);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!(
                                    "[Flow Edge] X11 query pointer error: {}. Reconnecting...",
                                    e
                                );
                                break;
                            }
                        }
                    }

                    std::thread::sleep(Duration::from_millis(50));
                }
            }
            Err(e) => {
                log::debug!(
                    "[Flow Edge] X11 connection failed: {}. Retrying in 5s...",
                    e
                );
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    }
}
