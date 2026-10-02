use crate::lock_ext::MutexExt;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct DeviceStateUpdate {
    pub unifying_receiver_connected: bool,
    pub bolt_receiver_connected: bool,
    pub bluetooth_available: bool,
    pub paired_devices: std::sync::Arc<Vec<(String, String, bool)>>, // (mac, name, is_connected)
    pub battery_pct: String,
    pub battery_status: String,
    pub has_active_hidpp_battery: bool,
    pub active_profile: String,
}

#[allow(dead_code)]
pub enum BackgroundTxCmd {
    TriggerPoll,
    Unpair(String),
}

pub fn spawn_background_worker(
    repaint_callback: impl Fn() + Send + Sync + 'static,
    active_profile_ref: std::sync::Arc<std::sync::Mutex<String>>,
) -> (Sender<BackgroundTxCmd>, Receiver<DeviceStateUpdate>) {
    let (tx_cmd, rx_cmd) = channel::<BackgroundTxCmd>();
    let (tx_state, rx_state) = channel::<DeviceStateUpdate>();

    std::thread::spawn(move || {
        let mut last_poll = None;
        let mut paired_devices = crate::cache::load_device_cache();

        loop {
            // Check commands with a short timeout to be responsive to user actions
            let mut force_poll = false;

            // Wait with a small timeout so the thread loop checks for polling interval periodically
            match rx_cmd.recv_timeout(Duration::from_millis(200)) {
                Ok(BackgroundTxCmd::TriggerPoll) => {
                    force_poll = true;
                }
                Ok(BackgroundTxCmd::Unpair(mac)) => {
                    // Call bluetooth unpair — only remove from local cache if it
                    // actually succeeded at the bluez level.  If the remove failed
                    // (e.g. device already gone, bluez not settled) we must NOT
                    // delete the local entry or the cache, otherwise a successful
                    // re-pair later will collide with the stale cache entry and
                    // produce duplicate device cards in the GUI.
                    if crate::bluetooth::unpair_device(&mac) {
                        paired_devices.retain(|(m, _, _)| m != &mac);
                        // Also prune any matching entry from the on-disk cache so a
                        // restart cannot resurrect the just-unpaired device.
                        crate::cache::save_device_cache(&paired_devices);
                    } else {
                        log::warn!(
                            "[worker] bluetoothctl remove {} failed — keeping local entry",
                            mac
                        );
                    }
                    force_poll = true;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Timeout hit - regular interval check
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // Main app exited
                    break;
                }
            }

            let now = Instant::now();
            let is_connected = paired_devices.iter().any(|(_, _, c)| *c);
            let poll_interval = if is_connected {
                Duration::from_secs(5)
            } else {
                Duration::from_secs(2) // Polling rate of 2 seconds when disconnected
            };

            let should_poll = force_poll
                || match last_poll {
                    Some(last) => now.duration_since(last) >= poll_interval,
                    None => true,
                };

            if should_poll {
                let (unifying, bolt) = crate::receiver::detect_receivers();
                let (fresh_devices, bt_up) = crate::bluetooth::get_paired_logitech_devices();

                if bt_up {
                    let mut changed = false;
                    // Build a map of fresh devices by MAC for easy lookup
                    let fresh_map: std::collections::HashMap<String, (String, bool)> =
                        fresh_devices
                            .iter()
                            .map(|(m, n, c)| (m.clone(), (n.clone(), *c)))
                            .collect();

                    // Update existing cached devices and prune stale ones.
                    //
                    // Prune rule: if a cached device is absent from bluetoothctl's
                    // "devices Paired" output AND is already marked disconnected,
                    // remove it entirely.  A device that was *just* disconnected on
                    // the previous poll might reappear on this poll if the OS-level
                    // unpair hasn't landed yet — keeping it disconnected for one extra
                    // poll cycle avoids deleting devices that are in the process of
                    // being unpaired/re-paired.  Devices that are absent and already
                    // disconnected have been gone for at least one full poll cycle and
                    // are safe to prune.
                    let mut i = 0;
                    while i < paired_devices.len() {
                        // Read fields by value so we do not hold an immutable borrow
                        // across the mutable writes below — Rust's borrow checker
                        // forbids mutating paired_devices[i] while &paired_devices[i]
                        // is still live.
                        let mac = paired_devices[i].0.clone();
                        let name = paired_devices[i].1.clone();
                        let connected = paired_devices[i].2;

                        if let Some((fresh_name, fresh_conn)) = fresh_map.get(&mac) {
                            if name != *fresh_name {
                                paired_devices[i].1 = fresh_name.clone();
                                changed = true;
                            }
                            if connected != *fresh_conn {
                                paired_devices[i].2 = *fresh_conn;
                                changed = true;
                            }
                        } else if !connected {
                            // Absent from fresh paired list AND already disconnected →
                            // prune the stale entry (and the cache will be updated below).
                            paired_devices.remove(i);
                            changed = true;
                            // do not increment i — next element shifts into this index
                        } else {
                            // Absent from fresh list but still marked connected →
                            // mark disconnected but keep for one more poll cycle.
                            paired_devices[i].2 = false;
                            changed = true;
                            i += 1;
                        }
                    }

                    // Add newly paired devices to cache
                    for (mac, name, connected) in fresh_devices {
                        if !paired_devices.iter().any(|(m, _, _)| m == &mac) {
                            paired_devices.push((mac.clone(), name.clone(), connected));
                            changed = true;
                        }
                    }

                    if changed {
                        crate::cache::save_device_cache(&paired_devices);
                    }
                } else {
                    // Bluetooth is down - mark all cached devices as disconnected
                    for (_, _, connected) in &mut paired_devices {
                        if *connected {
                            *connected = false;
                        }
                    }
                }

                // Query battery first over direct HID++ (OpenLogi approach — richer
                // status), falling back to sysfs when no device is reachable
                // from this thread.
                let has_active_hidpp = crate::battery::has_active_hidpp_battery();
                let (battery_pct, battery_status) =
                    if paired_devices.iter().any(|(_, _, c)| *c) || has_active_hidpp {
                        match crate::battery::get_mouse_battery_hidpp() {
                            Some((pct, status)) => (pct, status),
                            None => match crate::battery::get_mouse_battery() {
                                Some((_, pct)) => (pct, String::new()),
                                None => ("80".to_string(), String::new()),
                            },
                        }
                    } else {
                        ("0".to_string(), String::new())
                    };

                let active_profile = active_profile_ref.lock_safe().clone();
                let update = DeviceStateUpdate {
                    unifying_receiver_connected: unifying,
                    bolt_receiver_connected: bolt,
                    bluetooth_available: bt_up,
                    paired_devices: std::sync::Arc::new(paired_devices.clone()),
                    battery_pct,
                    battery_status,
                    has_active_hidpp_battery: has_active_hidpp,
                    active_profile,
                };

                let _ = tx_state.send(update);
                repaint_callback(); // Wake up GUI loop to process the new state
                last_poll = Some(now);
            }
        }
    });

    (tx_cmd, rx_state)
}
