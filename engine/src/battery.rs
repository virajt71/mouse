use crate::hidpp::HidppClient;

/// Poll battery directly over HID++ (OpenLogi approach): open the device,
/// probe unified (0x1004) → legacy (0x1000) → voltage (0x1001) in priority
/// order, read the percentage + a status label. Returns None when no HID++
/// device is reachable.
pub fn get_mouse_battery_hidpp() -> Option<(String, String)> {
    let mut client = HidppClient::default();
    client.open_for_battery().ok()?;
    client.read_battery().map(|b| (b.percentage.to_string(), b.status))
}

pub fn get_mouse_battery() -> Option<(String, String)> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with("hidpp_battery_") {
                    let path = entry.path();
                    let level_path = path.join("capacity_level");
                    let cap_path = path.join("capacity");

                    if let Ok(pct) = std::fs::read_to_string(cap_path) {
                        let pct_trim = pct.trim().to_string();
                        if !pct_trim.is_empty() {
                            return Some((format!("{}%", pct_trim), pct_trim));
                        }
                    }

                    if let Ok(level) = std::fs::read_to_string(level_path) {
                        let lvl_trim = level.trim().to_string();
                        if !lvl_trim.is_empty() {
                            let pct_str = match lvl_trim.to_lowercase().as_str() {
                                "full" => "100",
                                "high" => "80",
                                "normal" => "50",
                                "low" => "20",
                                "critical" => "5",
                                _ => "80",
                            };
                            return Some((lvl_trim, pct_str.to_string()));
                        }
                    }
                }
            }
        }
    }

    // Mock for other platforms
    Some(("Full".to_string(), "100".to_string()))
}

pub fn has_active_hidpp_battery() -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("hidpp_battery_")
                {
                    return true;
                }
            }
        }
    }
    false
}