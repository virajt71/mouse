// Returns (devices, bluetooth_was_reachable).
// bluetooth_was_reachable = false means the BT adapter is off/unavailable,
// so the caller should keep its cached device list intact.
pub fn get_paired_logitech_devices() -> (Vec<(String, String, bool)>, bool) {
    let mut devices = Vec::new();

    #[cfg(target_os = "linux")]
    {
        let result = std::process::Command::new("bluetoothctl")
            .arg("devices")
            .arg("Paired")
            .output();

        match result {
            Err(_) => {
                // bluetoothctl not found or couldn't spawn — treat BT as unavailable
                (devices, false)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);

                // If stderr contains "not available" or "No default controller"
                // the Bluetooth adapter is powered off.
                if stderr.contains("not available")
                    || stderr.contains("No default controller")
                    || stdout.contains("not available")
                {
                    (devices, false)
                } else {
                    for line in stdout.lines() {
                        // Output format: Device XX:XX:XX:XX:XX:XX Device Name
                        let parts: Vec<&str> = line.splitn(3, ' ').collect();
                        if parts.len() == 3 && parts[0] == "Device" {
                            let mac = parts[1].to_string();
                            let name = parts[2].trim().to_string();
                            let name_lower = name.to_lowercase();
                            // Match Logitech or MX products
                            if name_lower.contains("logitech")
                                || name_lower.contains("logi")
                                || name_lower.contains("mx ")
                            {
                                let connected = is_device_connected(&mac);
                                devices.push((mac, name, connected));
                            }
                        }
                    }

                    (devices, true)
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

pub fn is_device_connected(mac: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("bluetoothctl")
            .arg("info")
            .arg(mac)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("Connected:") {
                    return trimmed.contains("yes");
                }
            }
        }
    }
    let _ = mac;
    false
}

pub fn unpair_device(mac: &str) {
    #[cfg(target_os = "linux")]
    {
        // Execute: bluetoothctl remove <mac> to unpair the device
        let _ = std::process::Command::new("bluetoothctl")
            .arg("remove")
            .arg(mac)
            .spawn();
    }
    let _ = mac;
}
