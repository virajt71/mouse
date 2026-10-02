use crate::hidpp::HidppClient;


/// True when a battery reading of 0% with this sysfs-derived status should be
/// treated as a cold start rather than "dead". Matches OpenLogi's rule:
/// charging + 0% = cold start, don't render as empty/depleted.
fn battery_cold_start_zero(status: &str) -> bool {
    status.starts_with("charging") || status == "recharging"
}

/// Poll battery directly over HID++ (OpenLogi approach): open the device,
/// probe unified (0x1004) → legacy (0x1000) → voltage (0x1001) in priority
/// order, read the percentage + a status label. Returns None when no HID++
/// device is reachable.
pub fn get_mouse_battery_hidpp() -> Option<(String, String)> {
    let mut client = HidppClient::default();
    client.open_for_battery().ok()?;
    client
        .read_battery()
        .map(|b| (b.percentage.to_string(), b.status))
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

                    // Determine charging status from the power supply type.
                    // AC = charger connected, USB = charging, Wireless = charging,
                    // Battery = discharging.  Default to discharging if unknown.
                    let status = if name == "AC" || name == "ACAD" {
                        "charging".to_string()
                    } else if name.starts_with("USB") || name.starts_with("wireless") {
                        "charging".to_string()
                    } else if name == "Battery" {
                        "discharging".to_string()
                    } else {
                        // Fallback: check online attribute if present
                        let online_path = path.join("online");
                        if let Ok(online) = std::fs::read_to_string(&online_path) {
                            if online.trim() == "1" {
                                "charging".to_string()
                            } else {
                                "discharging".to_string()
                            }
                        } else {
                            "discharging".to_string()
                        }
                    };

                    if let Ok(pct) = std::fs::read_to_string(cap_path) {
                        let pct_trim = pct.trim().to_string();
                        if !pct_trim.is_empty() {
                            // Cold-start guard: a Logitech HID++ battery under charge can
                            // report 0% until the firmware has a real reading. Don't show 0%
                            // (looks dead) — surface the charging status with no numeric claim
                            // so the UI can render "charging" without a misleading 0%.
                            if pct_trim == "0" && battery_cold_start_zero(&status) {
                                return Some((String::new(), status));
                            }
                            return Some((format!("{}%", pct_trim), status));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_start_zero_is_charging_statuses() {
        assert!(battery_cold_start_zero("charging"));
        assert!(battery_cold_start_zero("charging_slow"));
        assert!(battery_cold_start_zero("recharging"));
        assert!(battery_cold_start_zero("charging_nearly_full"));
        assert!(!battery_cold_start_zero("discharging"));
        assert!(!battery_cold_start_zero("full"));
        assert!(!battery_cold_start_zero(""));
        assert!(!battery_cold_start_zero("unknown"));
    }

    #[test]
    fn cold_start_zero_not_discharging() {
        assert!(!battery_cold_start_zero("discharging"));
    }
}
