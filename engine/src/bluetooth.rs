// Returns (devices, bluetooth_was_reachable).
// bluetooth_was_reachable = false means the BT adapter is off/unavailable,
// so the caller should keep its cached device list intact.
pub fn get_paired_logitech_devices() -> (Vec<(String, String, bool)>, bool) {
    let mut devices = Vec::new();

    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let result = Command::new("bluetoothctl")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match result {
            Err(_) => {
                // bluetoothctl not found or couldn't spawn — treat BT as unavailable
                (devices, false)
            }
            Ok(mut child) => {
                // Write both commands to stdin and exit
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(b"devices Paired\ndevices Connected\nexit\n");
                }

                match child.wait_with_output() {
                    Err(_) => (devices, false),
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);

                        if stderr.contains("No default controller")
                            || stdout.contains("No default controller")
                        {
                            (devices, false)
                        } else {
                            let mut paired_list = Vec::new();
                            let mut connected_macs = std::collections::HashSet::new();
                            let mut parsing_connected = false;

                            for line in stdout.lines() {
                                if line.contains("devices Connected") {
                                    parsing_connected = true;
                                    continue;
                                } else if line.contains("devices Paired") {
                                    parsing_connected = false;
                                    continue;
                                }

                                // Output format: Device XX:XX:XX:XX:XX:XX Device Name
                                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                                if parts.len() >= 3 && parts[0] == "Device" {
                                    let mac = parts[1].to_string();
                                    let name = parts[2].trim().to_string();
                                    let name_lower = name.to_lowercase();
                                    // Match Logitech or MX products
                                    if name_lower.contains("logitech")
                                        || name_lower.contains("logi")
                                        || name_lower.contains("mx ")
                                    {
                                        if parsing_connected {
                                            connected_macs.insert(mac.to_uppercase());
                                        } else {
                                            paired_list.push((mac, name));
                                        }
                                    }
                                } else if parts.len() == 2 && parts[0] == "Device" {
                                    let mac = parts[1].to_string();
                                    if parsing_connected {
                                        connected_macs.insert(mac.to_uppercase());
                                    }
                                }
                            }

                            for (mac, name) in paired_list {
                                let connected = connected_macs.contains(&mac.to_uppercase());
                                devices.push((mac, name, connected));
                            }

                            (devices, true)
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Mock a paired device on non-Linux systems for development/UI presentation
        devices.push((
            "00:11:22:33:44:55".to_string(),
            "MX Master 3S".to_string(),
            false,
        ));
        return (devices, true);
    }
}

fn get_connected_bluetooth_macs() -> std::collections::HashSet<String> {
    let mut connected = std::collections::HashSet::new();
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("bluetoothctl")
            .args(["devices", "Connected"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                if parts.len() >= 2 && parts[0] == "Device" {
                    connected.insert(parts[1].to_uppercase());
                }
            }
        }
    }
    connected
}

pub fn is_device_connected(mac: &str) -> bool {
    let connected = get_connected_bluetooth_macs();
    connected.contains(&mac.to_uppercase())
}

/// Remove a paired Bluetooth device from bluez.
///
/// Validates the MAC address format before calling bluetoothctl so we fail
/// fast on corrupted identifiers (which would otherwise send garbage to
/// bluez and silently "succeed" because bluetoothctl accepts any string as
/// a device argument).
pub fn unpair_device(mac: &str) -> bool {
    // Normalize and validate MAC format before calling bluez. A malformed MAC
    // means the caller passed garbage (e.g. a stale cache entry with a
    // corrupted identifier); there is nothing to unpair.
    let normalized = mac.to_uppercase();
    if !is_valid_bt_mac(&normalized) {
        log::warn!(
            "[bluetooth] refusing to unpair invalid MAC address: {}",
            mac
        );
        return false;
    }

    #[cfg(target_os = "linux")]
    {
        // bluetoothctl 'remove' fails if the device is currently connected —
        // bluez requires a 'disconnect' first. Interactive-mode bluetoothctl
        // handles this internally, but the one-shot 'bluetoothctl remove MAC'
        // subcommand does not, so we must disconnect explicitly before removing.
        // Run both commands inside a single bluetoothctl session so the state
        // changes are processed atomically by bluetoothd.
        use std::io::Write;
        use std::process::{Command, Stdio};

        let result = Command::new("bluetoothctl")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();

        match result {
            Err(e) => {
                log::warn!("[bluetooth] failed to spawn bluetoothctl for unpair {}: {}", mac, e);
                false
            }
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    // Disconnect first (ignored if already disconnected), then remove.
                    // 'exit' closes the session so bluetoothctl processes both commands
                    // and terminates — without it the child would hang waiting for more input.
                    let _ = stdin.write_all(
                        format!("disconnect {}; remove {}; exit
", normalized, normalized).as_bytes(),
                    );
                }
                match child.wait_with_output() {
                    Err(e) => {
                        log::warn!("[bluetooth] bluetoothctl unpair session failed for {}: {}", mac, e);
                        false
                    }
                    Ok(out) => {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        // If stderr contains a fatal error (e.g. "No default controller"),
                        // the unpair definitely did not succeed.
                        if stderr.contains("No default controller") {
                            log::warn!("[bluetooth] bluetoothctl unpair failed for {}: no default controller", mac);
                            false
                        } else if !out.status.success() {
                            log::warn!(
                                "[bluetooth] bluetoothctl unpair {} failed (exit {}): {}",
                                mac,
                                out.status,
                                stderr
                            );
                            false
                        } else {
                            true
                        }
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = normalized;
        true
    }
}

/// Returns true if `mac` looks like a valid Bluetooth MAC address
/// (6 colon-separated hex octets, e.g. `AA:BB:CC:DD:EE:FF`).
#[inline]
pub fn is_valid_bt_mac(mac: &str) -> bool {
    let mut parts = mac.split(':');
    for _ in 0..6 {
        match parts.next() {
            Some(p) if p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()) => {}
            _ => return false,
        }
    }
    parts.next().is_none()
}
