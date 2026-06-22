use std::process::Stdio;
use tokio::io::AsyncWriteExt;

/// Validate that a string is a well-formed Bluetooth MAC address
/// (exactly six colon-separated hex octets: `XX:XX:XX:XX:XX:XX`).
/// Rejects anything else to prevent argument injection into `bluetoothctl`.
fn is_valid_mac(mac: &str) -> bool {
    let b = mac.as_bytes();
    if b.len() != 17 {
        return false;
    }
    for (i, &byte) in b.iter().enumerate() {
        if (i + 1) % 3 == 0 {
            if byte != b':' {
                return false;
            }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

// Returns (devices, bluetooth_was_reachable).
// bluetooth_was_reachable = false means the BT adapter is off/unavailable,
// so the caller should keep its cached device list intact.
pub async fn get_paired_logitech_devices() -> (Vec<(String, String, bool)>, bool) {
    let mut devices = Vec::new();

    #[cfg(target_os = "linux")]
    {
        let result = tokio::process::Command::new("bluetoothctl")
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
                    let _ = stdin.write_all(b"devices Paired\ndevices Connected\nexit\n").await;
                }

                match child.wait_with_output().await {
                    Err(_) => (devices, false),
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);

                        if stderr.contains("not available")
                            || stderr.contains("No default controller")
                            || stdout.contains("not available")
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

async fn get_connected_bluetooth_macs() -> std::collections::HashSet<String> {
    let mut connected = std::collections::HashSet::new();
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = tokio::process::Command::new("bluetoothctl")
            .args(["devices", "Connected"])
            .output()
            .await
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

pub async fn is_device_connected(mac: &str) -> bool {
    if !is_valid_mac(mac) {
        log::warn!("[Bluetooth] Rejected invalid MAC address: {:?}", mac);
        return false;
    }
    let connected = get_connected_bluetooth_macs().await;
    connected.contains(&mac.to_uppercase())
}

pub async fn unpair_device(mac: &str) {
    #[cfg(target_os = "linux")]
    {
        if !is_valid_mac(mac) {
            log::warn!("[Bluetooth] Rejected unpair request for invalid MAC address: {:?}", mac);
            return;
        }
        // Execute: bluetoothctl remove <mac> to unpair the device
        let _ = tokio::process::Command::new("bluetoothctl")
            .arg("remove")
            .arg(mac)
            .spawn();
    }
    let _ = mac;
}
