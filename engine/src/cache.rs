use std::path::PathBuf;

/// Returns the path to the persistent paired-device cache file.
/// Location: ~/.local/share/mouser-rs/paired_devices.csv
pub fn device_cache_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("MOUSER_DEVICE_CACHE_PATH") {
        return Some(PathBuf::from(p));
    }
    let mut path = dirs::data_local_dir()?; // ~/.local/share on Linux
    path.push("mouser-rs");
    path.push("paired_devices.csv");
    Some(path)
}

/// Persist the in-memory device list to disk.
/// Format: one device per line — `mac,name` (connection state is never saved;
/// it is always re-evaluated at runtime).
pub fn save_device_cache(devices: &[(String, String, bool)]) {
    let Some(path) = device_cache_path() else {
        return;
    };

    let content: String = devices
        .iter()
        .map(|(mac, name, _)| format!("{},{}\n", mac, name.replace(',', " ")))
        .collect();

    // Check if on-disk content matches what we are about to write
    if let Ok(existing_content) = std::fs::read_to_string(&path) {
        if existing_content == content {
            return; // Cache is clean, skip disk write
        }
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let _ = std::fs::write(&path, content);
}

/// Load the previously persisted device list from disk.
/// All devices are returned with `is_connected = false`; live state is filled
/// in by the poll loop once BT comes back online.
pub fn load_device_cache() -> Vec<(String, String, bool)> {
    let Some(path) = device_cache_path() else {
        return Vec::new();
    };

    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };

    content
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ',');
            let mac = parts.next()?.trim().to_string();
            let name = parts.next()?.trim().to_string();
            if mac.is_empty() || name.is_empty() {
                return None;
            }
            Some((mac, name, false)) // connection state always starts as disconnected
        })
        .collect()
}
