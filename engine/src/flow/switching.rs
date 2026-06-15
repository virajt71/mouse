use super::FLOW_MANAGER;
use std::sync::Arc;
use std::time::Duration;
use x11rb::connection::Connection;

pub fn run_edge_detection_loop(engine_inner: Arc<crate::engine::inner::EngineInner>) {
    // Dedicated edge-detection thread (primarily for X11/XWayland active pointer query fallback)
    // Wayland uses raw evdev injection from MouseHook, but X11 can run query_pointer
    use std::sync::atomic::Ordering;

    loop {
        if !engine_inner.running.load(Ordering::SeqCst) {
            break;
        }

        let enabled = FLOW_MANAGER.flow_enabled.load(Ordering::SeqCst);

        if !enabled {
            std::thread::sleep(Duration::from_millis(1000));
            continue;
        }

        let x11_conn = x11rb::rust_connection::RustConnection::connect(None);
        match x11_conn {
            Ok((conn, screen_num)) => {
                log::info!("[Flow Switching] X11 QueryPointer edge detection running...");
                let screen = &conn.setup().roots[screen_num];
                let root = screen.root;

                loop {
                    if !engine_inner.running.load(Ordering::SeqCst) {
                        return;
                    }

                    let (inner_enabled, hold_key) = (
                        FLOW_MANAGER.flow_enabled.load(Ordering::SeqCst),
                        FLOW_MANAGER.flow_hold_key.read().unwrap().clone(),
                    );

                    if !inner_enabled {
                        break;
                    }

                    if FLOW_MANAGER.get_active_peer_name().is_none() {
                        // If using software mode and mouse is local, check pointer position
                        use x11rb::protocol::xproto::ConnectionExt;
                        match conn.query_pointer(root) {
                            Ok(cookie) => {
                                match cookie.reply() {
                                    Ok(reply) => {
                                        let rx = reply.root_x as i32;
                                        let ry = reply.root_y as i32;
                                        let sw = screen.width_in_pixels as i32;
                                        let sh = screen.height_in_pixels as i32;

                                        // Update local screen dimensions in FlowManager
                                        *FLOW_MANAGER.screen_width.write().unwrap() = sw;
                                        *FLOW_MANAGER.screen_height.write().unwrap() = sh;

                                        let threshold = 2;
                                        let mut lx = 0;
                                        let mut ly = 0;

                                        if rx <= threshold {
                                            lx = -1;
                                        } else if rx >= sw - threshold - 1 {
                                            lx = 1;
                                        } else if ry <= threshold {
                                            ly = -1;
                                        } else if ry >= sh - threshold - 1 {
                                            ly = 1;
                                        }

                                        if lx != 0 || ly != 0 {
                                            // Check if transition modifier key is satisfied
                                            if is_hold_key_satisfied(&hold_key) {
                                                let peers = FLOW_MANAGER.flow_peers.read().unwrap();
                                                if let Some(peer) =
                                                    peers.iter().find(|p| {
                                                        p.paired
                                                            && p.layout_x == lx
                                                            && p.layout_y == ly
                                                    })
                                                {
                                                    // If this peer is currently controlling us, return control to them!
                                                    if let Some(controller) =
                                                        FLOW_MANAGER.get_current_controller()
                                                    {
                                                        if controller == peer.name {
                                                            log::info!("[Flow Switching] Screen edge crossed on X11: returning to controller '{}'", peer.name);
                                                            let _ = crate::flow::network::send_event_to_peer(&peer.name, &crate::flow::network::FlowEvent::ReturnToLocal);
                                                            // Warp cursor away from edge to prevent loop bouncing
                                                            let rx_target = if lx == -1 {
                                                                100
                                                            } else if lx == 1 {
                                                                sw - 100
                                                            } else {
                                                                rx
                                                            };
                                                            let ry_target = if ly == -1 {
                                                                100
                                                            } else if ly == 1 {
                                                                sh - 100
                                                            } else {
                                                                ry
                                                            };
                                                            use x11rb::protocol::xproto::ConnectionExt;
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
                                                            continue;
                                                        }
                                                    }

                                                    log::info!("[Flow Switching] Screen edge crossed on X11: transitioning to peer '{}'", peer.name);
                                                    FLOW_MANAGER
                                                        .set_active_peer(Some(peer.name.clone()));
                                                    // Sync virtual coords for the transition
                                                    *FLOW_MANAGER.virtual_x.lock().unwrap() =
                                                        if lx == -1 { sw - 50 } else { 50 };

                                                    let mode = FLOW_MANAGER.flow_mouse_mode.read().unwrap().clone();
                                                    if mode == "hardware" {
                                                        if let Some(idx) = crate::flow::switching::get_peer_channel_index(&peer.name) {
                                                            crate::flow::switching::trigger_hidpp_channel_switch(idx);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("[Flow Switching] X11 query reply error: {}. Reconnecting X11...", e);
                                        break; // Break inner loop to reconnect
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("[Flow Switching] X11 query pointer error: {}. Reconnecting X11...", e);
                                break; // Break inner loop to reconnect
                            }
                        }
                    }

                    std::thread::sleep(Duration::from_millis(50));
                }
            }
            Err(e) => {
                log::debug!(
                    "[Flow Switching] X11 connection failed: {}. Retrying in 5s...",
                    e
                );
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    }
}

pub fn is_hold_key_satisfied(hold_key: &str) -> bool {
    if hold_key == "none" {
        return true;
    }
    // Query modifier keys state from keyboard hook
    // In this implementation, we return true to allow transition.
    // If keyboard hook exposes modifiers, we can hook it.
    true
}

pub fn get_peer_channel_index(peer_name: &str) -> Option<u8> {
    let peers = FLOW_MANAGER.flow_peers.read().unwrap();
    peers.iter()
        .find(|p| p.name == peer_name)
        .map(|p| p.channel_index)
}

pub fn trigger_hidpp_channel_switch(channel_idx: u8) {
    log::info!(
        "[Flow Switching] Triggering HID++ host switch to channel {}",
        channel_idx
    );
    if let Some(ref inner) = *FLOW_MANAGER.engine_inner.lock().unwrap() {
        let mut clients = inner.hid_clients.lock().unwrap();
        for client in clients.iter_mut() {
            if client.change_host_idx.is_some() {
                if let Err(e) = client.switch_host_channel(channel_idx) {
                    log::error!(
                        "[Flow Switching] Failed to switch channel on device '{}': {}",
                        client.device_name,
                        e
                    );
                }
            }
        }
    }
}
