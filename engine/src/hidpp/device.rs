use super::protocol::{
    FEAT_ADJ_DPI, FEAT_BACKLIGHT2, FEAT_CHANGE_HOST, FEAT_REPROG_V4, FEAT_SMART_SHIFT,
    FEAT_SMART_SHIFT_ENHANCED, LOGI_VID,
};
use super::HidppClient;
use anyhow::{anyhow, Result};
use hidapi::HidApi;

impl HidppClient {
    pub fn open_device(&mut self) -> Result<()> {
        let api = HidApi::new()?;
        let devices = self.list_hidpp_devices(&api);
        let info = devices
            .first()
            .ok_or_else(|| anyhow!("No Logitech HID++ device found"))?;
        self.open_path(&api, info.path())
    }

    pub fn list_hidpp_devices(&self, api: &HidApi) -> Vec<hidapi::DeviceInfo> {
        let mut list = Vec::new();
        for device in api.device_list() {
            if device.vendor_id() == LOGI_VID {
                let usage_page = device.usage_page();
                if usage_page >= 0xFF00
                    || device.product_id() == 0xC548
                    || device.product_id() == 0xC52B
                {
                    list.push(device.clone());
                }
            }
        }
        list
    }

    pub fn open_path(&mut self, api: &HidApi, path: &std::ffi::CStr) -> Result<()> {
        let info = api
            .device_list()
            .find(|d| d.path() == path)
            .ok_or_else(|| anyhow!("Device path not found"))?;

        let product_id = info.product_id();
        let product_name = info.product_string().unwrap_or("Unknown").to_string();

        log::debug!(
            "[HID++] Opening device: {} (PID: 0x{:04X})",
            product_name,
            product_id
        );

        // Map known Product IDs to layout keys
        let layout_from_pid = match product_id {
            0x4082 => Some("mx_master_3"),
            0x4091 => Some("mx_master_3s"),
            0x406B => Some("mx_anywhere_2s"),
            0x4090 => Some("mx_anywhere_3"),
            0x40A3 => Some("mx_anywhere_3s"),
            0x408A => Some("mx_vertical"),
            0x4069 => Some("mx_ergo"),
            0x4072 => Some("mx_keys"),
            0x4093 => Some("mx_keys_s"),
            0x408D => Some("mx_keys_mini"),
            0x408E => Some("mx_mechanical"),
            0x408F => Some("mx_mechanical_mini"),
            _ => None,
        };

        let dev = api.open_path(path)?;
        self.device = Some(dev);
        self.device_name = product_name;
        self.device_path = path.to_string_lossy().to_string();
        self.layout_from_pid = layout_from_pid;

        // Reset state
        self.feat_idx = None;
        self.dpi_idx = None;
        self.smart_shift_idx = None;
        self.change_host_idx = None;
        self.backlight_feat_idx = None;
        self.held = false;
        self.mode_shift_held = false;

        // Probe dev_idx from direct Bluetooth (0xFF) and receiver slots (1 to 6)
        let mut found = false;
        for idx in [0xFF, 1, 2, 3, 4, 5, 6] {
            self.dev_idx = idx;
            if let Some(feat_idx) = self.find_feature(FEAT_REPROG_V4) {
                self.feat_idx = Some(feat_idx);
                log::info!(
                    "[HID++] Found REPROG_V4 at index 0x{:02X} for dev_idx 0x{:02X}",
                    feat_idx,
                    idx
                );
                found = true;
                break;
            }
        }

        if !found {
            self.device = None;
            return Err(anyhow!("Failed to locate active HID++ feature index"));
        }

        // Query secondary features
        self.dpi_idx = self.find_feature(FEAT_ADJ_DPI);
        if let Some(dpi_fi) = self.dpi_idx {
            log::info!("[HID++] Found ADJUSTABLE_DPI at index 0x{:02X}", dpi_fi);
        }

        if let Some(ss_fi) = self.find_feature(FEAT_SMART_SHIFT_ENHANCED) {
            self.smart_shift_idx = Some(ss_fi);
            self.smart_shift_enhanced = true;
            log::info!(
                "[HID++] Found SMART_SHIFT_ENHANCED at index 0x{:02X}",
                ss_fi
            );
        } else if let Some(ss_fi) = self.find_feature(FEAT_SMART_SHIFT) {
            self.smart_shift_idx = Some(ss_fi);
            self.smart_shift_enhanced = false;
            log::info!("[HID++] Found SMART_SHIFT at index 0x{:02X}", ss_fi);
        }

        self.change_host_idx = self.find_feature(FEAT_CHANGE_HOST);
        if let Some(ch_fi) = self.change_host_idx {
            log::info!("[HID++] Found CHANGE_HOST at index 0x{:02X}", ch_fi);
        }

        self.backlight_feat_idx = self.find_feature(FEAT_BACKLIGHT2);
        if let Some(bl_fi) = self.backlight_feat_idx {
            log::info!("[HID++] Found BACKLIGHT2 (0x1982) at index 0x{:02X}", bl_fi);
        }

        // Program button diversion (only for non-keyboards/devices with gesture support)
        self.held = false;
        self.mode_shift_held = false;
        let layout_key = self.get_layout_key();
        if !layout_key.starts_with("mx_keys") && !layout_key.starts_with("mx_mechanical") {
            let _ = self.divert_gesture_button();
        }

        Ok(())
    }

    pub fn get_layout_key(&self) -> String {
        // First try to identify by Product ID
        if let Some(pid_layout) = self.layout_from_pid {
            return pid_layout.to_string();
        }

        get_layout_key_from_name(&self.device_name)
    }
}

pub fn get_layout_key_from_name(name: &str) -> String {
    let name = name.to_lowercase();
    if name.contains("mx mechanical mini") {
        "mx_mechanical_mini".to_string()
    } else if name.contains("mx mechanical")
        || name.contains("mechanical")
        || name.contains("mchncl")
    {
        "mx_mechanical".to_string()
    } else if name.contains("master 3s") {
        "mx_master_3s".to_string()
    } else if name.contains("master 3") || name.contains("master 4") {
        "mx_master_3".to_string()
    } else if name.contains("master 2") || name.contains("master 2s") {
        "mx_master_2s".to_string()
    } else if name.contains("master") {
        "mx_master".to_string()
    } else if name.contains("anywhere 3s") {
        "mx_anywhere_3s".to_string()
    } else if name.contains("anywhere 3") {
        "mx_anywhere_3".to_string()
    } else if name.contains("anywhere") {
        "mx_anywhere".to_string()
    } else if name.contains("vertical") {
        "mx_vertical".to_string()
    } else if name.contains("ergo") {
        "mx_ergo".to_string()
    } else if name.contains("mx keys mini") {
        "mx_keys_mini".to_string()
    } else if name.contains("mx keys s") {
        "mx_keys_s".to_string()
    } else if name.contains("mx keys") {
        "mx_keys".to_string()
    } else {
        "generic".to_string()
    }
}

impl HidppClient {
    pub fn switch_host_channel(&self, channel_index: u8) -> Result<()> {
        let idx = self
            .change_host_idx
            .ok_or_else(|| anyhow!("ChangeHost feature not supported on this device"))?;
        let resp = self.request(idx, 1, &[channel_index], 1000)?;
        if resp.is_some() {
            log::info!(
                "[HID++] Command to change host to channel {} sent successfully",
                channel_index
            );
            Ok(())
        } else {
            Err(anyhow!("Change host command failed"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_layout_key_from_name() {
        assert_eq!(
            get_layout_key_from_name("MX Mechanical Mini"),
            "mx_mechanical_mini"
        );
        assert_eq!(
            get_layout_key_from_name("MX Mechanical Keyboard"),
            "mx_mechanical"
        );
        assert_eq!(
            get_layout_key_from_name("Logitech Mechanical"),
            "mx_mechanical"
        );
        assert_eq!(get_layout_key_from_name("MX Keys Mini"), "mx_keys_mini");
        assert_eq!(get_layout_key_from_name("MX Keys S"), "mx_keys_s");
        assert_eq!(get_layout_key_from_name("MX Keys"), "mx_keys");
        assert_eq!(get_layout_key_from_name("MX Master 3S"), "mx_master_3s");
        assert_eq!(get_layout_key_from_name("MX Anywhere 3"), "mx_anywhere_3");
        assert_eq!(get_layout_key_from_name("Some Generic Keyboard"), "generic");
    }

    #[test]
    fn test_feat_change_host_value() {
        assert_eq!(crate::hidpp::protocol::FEAT_CHANGE_HOST, 0x1814);
    }
}
