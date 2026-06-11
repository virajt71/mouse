use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct DeviceStateUpdate {
    pub unifying_receiver_connected: bool,
    pub bolt_receiver_connected: bool,
    pub bluetooth_available: bool,
    pub paired_devices: Vec<(String, String, bool)>, // (mac, name, is_connected)
    pub battery_pct: String,
    pub has_active_hidpp_battery: bool,
    pub active_profile: String,
    pub device_batteries: std::collections::HashMap<String, String>,
    pub device_conn_types: std::collections::HashMap<String, String>,
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
                    // Call bluetooth unpair and remove from local paired list immediately
                    crate::bluetooth::unpair_device(&mac);
                    paired_devices.retain(|(m, _, _)| m != &mac);
                    crate::cache::save_device_cache(&paired_devices);
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
                let (fresh_devices, bt_up) =
                    crate::bluetooth::get_paired_logitech_devices();

                if bt_up {
                    let mut changed = false;
                    // Build a map of fresh devices by MAC for easy lookup
                    let fresh_map: std::collections::HashMap<String, (String, bool)> =
                        fresh_devices.iter().map(|(m, n, c)| (m.clone(), (n.clone(), *c))).collect();

                    // Update existing cached devices
                    for (mac, name, connected) in &mut paired_devices {
                        if let Some((fresh_name, fresh_conn)) = fresh_map.get(mac) {
                            if name != fresh_name {
                                *name = fresh_name.clone();
                                changed = true;
                            }
                            if *connected != *fresh_conn {
                                *connected = *fresh_conn;
                                changed = true;
                            }
                        } else {
                            // Device is in cache but not currently paired/present at OS level
                            if *connected {
                                *connected = false;
                                changed = true;
                            }
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

                let hidpp_devices = crate::battery::get_hidpp_devices();

                let mut device_batteries = std::collections::HashMap::new();
                let mut device_conn_types = std::collections::HashMap::new();
                let mut updated_paired_devices = paired_devices.clone();

                for (mac, name, connected) in &mut updated_paired_devices {
                    if let Some(dev_info) = crate::battery::match_device(mac, name, &hidpp_devices) {
                        *connected = true; // Actively connected via receiver or bluetooth
                        device_batteries.insert(mac.clone(), dev_info.battery_pct.clone());
                        let conn = if dev_info.is_bluetooth {
                            "bluetooth".to_string()
                        } else if bolt && (name.to_lowercase().contains("mechanical") || name.to_lowercase().contains("keys") || name.to_lowercase().contains("3s") || !unifying) {
                            "bolt".to_string()
                        } else if unifying {
                            "unifying".to_string()
                        } else {
                            "bluetooth".to_string()
                        };
                        device_conn_types.insert(mac.clone(), conn);
                    } else {
                        // Fallback logic
                        if *connected {
                            // Checked by bluetoothctl as connected, but no hidpp info
                            device_batteries.insert(mac.clone(), "80".to_string());
                            device_conn_types.insert(mac.clone(), "bluetooth".to_string());
                        } else {
                            device_batteries.insert(mac.clone(), "0".to_string());
                            device_conn_types.insert(mac.clone(), "bluetooth".to_string());
                        }
                    }
                }

                // Query general battery level (e.g. for mouse or legacy purposes)
                let has_active_hidpp = crate::battery::has_active_hidpp_battery();
                let battery_pct = if updated_paired_devices.iter().any(|(_, _, c)| *c) || has_active_hidpp {
                    crate::battery::get_mouse_battery()
                        .map(|(_, pct)| pct)
                        .unwrap_or_else(|| "80".to_string())
                } else {
                    "0".to_string()
                };

                let active_profile = active_profile_ref.lock().unwrap().clone();
                let update = DeviceStateUpdate {
                    unifying_receiver_connected: unifying,
                    bolt_receiver_connected: bolt,
                    bluetooth_available: bt_up,
                    paired_devices: updated_paired_devices,
                    battery_pct,
                    has_active_hidpp_battery: has_active_hidpp,
                    active_profile,
                    device_batteries,
                    device_conn_types,
                };

                let _ = tx_state.send(update);
                repaint_callback(); // Wake up GUI loop to process the new state
                last_poll = Some(now);
            }
        }
    });

    (tx_cmd, rx_state)
}
