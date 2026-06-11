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

#[derive(Debug, Clone)]
pub struct HidppDeviceInfo {
    pub mac: Option<String>,
    pub model_name: Option<String>,
    pub battery_pct: String,
    pub is_bluetooth: bool,
}

pub fn get_hidpp_devices() -> Vec<HidppDeviceInfo> {
    let mut devices = Vec::new();
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with("hidpp_battery_") {
                    let path = entry.path();
                    
                    let mut serial_number = None;
                    let mut model_name = None;
                    let mut is_bluetooth = true; // Default to Bluetooth, check bus below

                    if let Ok(uevent_content) = std::fs::read_to_string(path.join("uevent")) {
                        for line in uevent_content.lines() {
                            if let Some(val) = line.strip_prefix("POWER_SUPPLY_SERIAL_NUMBER=") {
                                serial_number = Some(val.trim().to_uppercase());
                            } else if let Some(val) = line.strip_prefix("POWER_SUPPLY_MODEL_NAME=") {
                                model_name = Some(val.trim().to_string());
                            }
                        }
                    }

                    // Check device bus type via HID_ID in device/uevent
                    if let Ok(dev_uevent) = std::fs::read_to_string(path.join("device/uevent")) {
                        for line in dev_uevent.lines() {
                            if let Some(val) = line.strip_prefix("HID_ID=") {
                                if val.starts_with("0003:") {
                                    is_bluetooth = false;
                                }
                            }
                        }
                    }

                    let mut battery_pct = "80".to_string();
                    let cap_path = path.join("capacity");
                    if let Ok(pct) = std::fs::read_to_string(cap_path) {
                        let pct_trim = pct.trim().to_string();
                        if !pct_trim.is_empty() {
                            battery_pct = pct_trim;
                        }
                    } else {
                        let level_path = path.join("capacity_level");
                        if let Ok(level) = std::fs::read_to_string(level_path) {
                            let lvl_trim = level.trim().to_string();
                            if !lvl_trim.is_empty() {
                                battery_pct = match lvl_trim.to_lowercase().as_str() {
                                    "full" => "100",
                                    "high" => "80",
                                    "normal" => "50",
                                    "low" => "20",
                                    "critical" => "5",
                                    _ => "80",
                                }.to_string();
                            }
                        }
                    }

                    devices.push(HidppDeviceInfo {
                        mac: serial_number,
                        model_name,
                        battery_pct,
                        is_bluetooth,
                    });
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Mock a connected device for UI testing
        devices.push(HidppDeviceInfo {
            mac: Some("00:11:22:33:44:55".to_string()),
            model_name: Some("MX Master 3S".to_string()),
            battery_pct: "95".to_string(),
            is_bluetooth: false,
        });
    }

    devices
}

pub fn match_device(mac: &str, name: &str, hidpp_devices: &[HidppDeviceInfo]) -> Option<HidppDeviceInfo> {
    let norm_mac = mac.replace(':', "").to_uppercase();
    let norm_name = name.to_lowercase();

    // 1. Try exact MAC match
    for dev in hidpp_devices {
        if let Some(ref dev_mac) = dev.mac {
            let norm_dev_mac = dev_mac.replace(':', "").to_uppercase();
            if !norm_dev_mac.is_empty() && norm_dev_mac == norm_mac {
                return Some(dev.clone());
            }
        }
    }

    // 2. Try substring name match
    for dev in hidpp_devices {
        if let Some(ref dev_model) = dev.model_name {
            let norm_model = dev_model.to_lowercase();
            if norm_model.contains(&norm_name) || norm_name.contains(&norm_model) {
                return Some(dev.clone());
            }

            let clean_model = norm_model.replace("wireless", "").replace("mouse", "").replace("keyboard", "").trim().to_string();
            let clean_name = norm_name.replace("wireless", "").replace("mouse", "").replace("keyboard", "").trim().to_string();
            
            // Abbreviation check for mechanical keyboard
            let clean_model = clean_model.replace("mechanical", "mchncl");
            let clean_name = clean_name.replace("mechanical", "mchncl");

            if !clean_model.is_empty() && !clean_name.is_empty() && (clean_model.contains(&clean_name) || clean_name.contains(&clean_model)) {
                return Some(dev.clone());
            }
        }
    }

    None
}

