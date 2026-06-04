use anyhow::{anyhow, Result};
use hidapi::{HidApi, HidDevice};
use std::time::{Duration, Instant};

const LOGI_VID: u16 = 0x046D;
const SHORT_ID: u8 = 0x10;
const LONG_ID: u8 = 0x11;
const LONG_LEN: usize = 20;
const MY_SW: u8 = 0x0A;

const FEAT_IROOT: u16 = 0x0000;
const FEAT_REPROG_V4: u16 = 0x1B04;
const FEAT_ADJ_DPI: u16 = 0x2201;
const FEAT_SMART_SHIFT: u16 = 0x2110;
const FEAT_SMART_SHIFT_ENHANCED: u16 = 0x2111;

pub enum HidppEvent {
    GestureDown,
    GestureUp,
    GestureMove { dx: i16, dy: i16 },
    ModeShiftDown,
    ModeShiftUp,
}

pub struct HidppClient {
    device: Option<HidDevice>,
    dev_idx: u8,
    feat_idx: Option<u8>,
    dpi_idx: Option<u8>,
    smart_shift_idx: Option<u8>,
    smart_shift_enhanced: bool,
    gesture_cid: u16,
    rawxy_enabled: bool,
    held: bool,
    mode_shift_held: bool,
    pub device_name: String,
    pub device_path: String,
}

impl Default for HidppClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HidppClient {
    pub fn new() -> Self {
        HidppClient {
            device: None,
            dev_idx: 0xFF,
            feat_idx: None,
            dpi_idx: None,
            smart_shift_idx: None,
            smart_shift_enhanced: false,
            gesture_cid: 0x00C3, // Default gesture button CID
            rawxy_enabled: false,
            held: false,
            mode_shift_held: false,
            device_name: "None".to_string(),
            device_path: "".to_string(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.device.is_some()
    }

    pub fn open_device(&mut self) -> Result<()> {
        let api = HidApi::new()?;
        let devices = self.list_hidpp_devices(&api);
        let info = devices.first().ok_or_else(|| anyhow!("No Logitech HID++ device found"))?;
        self.open_path(&api, info.path())
    }

    pub fn list_hidpp_devices(&self, api: &HidApi) -> Vec<hidapi::DeviceInfo> {
        let mut list = Vec::new();
        for device in api.device_list() {
            if device.vendor_id() == LOGI_VID {
                let usage_page = device.usage_page();
                if usage_page >= 0xFF00 || device.product_id() == 0xC548 || device.product_id() == 0xC52B {
                    list.push(device.clone());
                }
            }
        }
        list
    }

    pub fn open_path(&mut self, api: &HidApi, path: &std::ffi::CStr) -> Result<()> {
        let info = api.device_list().find(|d| d.path() == path)
            .ok_or_else(|| anyhow!("Device path not found"))?;

        let product_id = info.product_id();
        let product_name = info.product_string().unwrap_or("Unknown").to_string();

        log::debug!(
            "[HID++] Opening device: {} (PID: 0x{:04X})",
            product_name,
            product_id
        );

        let dev = api.open_path(path)?;
        self.device = Some(dev);
        self.device_name = product_name;
        self.device_path = path.to_string_lossy().to_string();

        // Reset state
        self.feat_idx = None;
        self.dpi_idx = None;
        self.smart_shift_idx = None;
        self.held = false;
        self.mode_shift_held = false;

        // Probe dev_idx from direct Bluetooth (0xFF) and receiver slots (1 to 6)
        let mut found = false;
        for idx in [0xFF, 1, 2, 3, 4, 5, 6] {
            self.dev_idx = idx;
            if let Some(feat_idx) = self.find_feature(FEAT_REPROG_V4) {
                self.feat_idx = Some(feat_idx);
                log::info!("[HID++] Found REPROG_V4 at index 0x{:02X} for dev_idx 0x{:02X}", feat_idx, idx);
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
            log::info!("[HID++] Found SMART_SHIFT_ENHANCED at index 0x{:02X}", ss_fi);
        } else if let Some(ss_fi) = self.find_feature(FEAT_SMART_SHIFT) {
            self.smart_shift_idx = Some(ss_fi);
            self.smart_shift_enhanced = false;
            log::info!("[HID++] Found SMART_SHIFT at index 0x{:02X}", ss_fi);
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

    pub fn close(&mut self) {
        if self.device.is_some() {
            // Restore default remapping before closing
            let _ = self.undivert_gesture_button();
        }
        self.device = None;
        self.device_name = "None".to_string();
        self.device_path = "".to_string();
        self.feat_idx = None;
        self.dpi_idx = None;
        self.smart_shift_idx = None;
    }

    fn tx(&self, feat: u8, func: u8, params: &[u8]) -> Result<()> {
        let dev = self.device.as_ref().ok_or_else(|| anyhow!("Device closed"))?;
        let mut buf = [0u8; LONG_LEN + 1]; // +1 for Report ID on some platforms, hidapi handles it
        buf[1] = LONG_ID;
        buf[2] = self.dev_idx;
        buf[3] = feat;
        buf[4] = ((func & 0x0F) << 4) | (MY_SW & 0x0F);

        for (i, &val) in params.iter().enumerate() {
            if i + 5 < buf.len() {
                buf[i + 5] = val;
            }
        }

        // On Linux/Windows/Mac, the write parameter should match report payload (first byte is report ID)
        dev.write(&buf[1..])?;
        Ok(())
    }

    fn rx(&self, timeout_ms: i32) -> Result<Option<Vec<u8>>> {
        let dev = self.device.as_ref().ok_or_else(|| anyhow!("Device closed"))?;
        let mut buf = [0u8; 64];
        let bytes_read = dev.read_timeout(&mut buf, timeout_ms)?;
        if bytes_read > 0 {
            Ok(Some(buf[..bytes_read].to_vec()))
        } else {
            Ok(None)
        }
    }

    fn request(&self, feat: u8, func: u8, params: &[u8], timeout_ms: u64) -> Result<Option<Vec<u8>>> {
        self.tx(feat, func, params)?;
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        let expected_func = ((func & 0x0F) << 4) | (MY_SW & 0x0F);

        while Instant::now() < deadline {
            let timeout = deadline.saturating_duration_since(Instant::now()).as_millis() as i32;
            if let Some(raw) = self.rx(timeout.min(100))? {
                if raw.len() < 4 {
                    continue;
                }
                // Determine layout offset
                let off = if raw[0] == SHORT_ID || raw[0] == LONG_ID { 1 } else { 0 };
                if off + 3 >= raw.len() {
                    continue;
                }

                let r_feat = raw[off + 1];
                let r_fsw = raw[off + 2];
                let r_params = &raw[off + 3..];

                // Error response
                if r_feat == 0xFF {
                    let err_code = if r_params.len() > 1 { r_params[1] } else { 0 };
                    // Error 0x02 is 'Unsupported' - log as debug instead of warn as it's common during probing
                    if err_code == 0x02 {
                        log::debug!("[HID++] Feature not supported (0x02)");
                    } else {
                        log::warn!("[HID++] Received error response: 0x{:02X}", err_code);
                    }
                    return Ok(None);
                }

                if r_feat == feat && r_fsw == expected_func {
                    return Ok(Some(r_params.to_vec()));
                }
            }
        }
        Ok(None)
    }

    fn find_feature(&self, feature_id: u16) -> Option<u8> {
        let hi = ((feature_id >> 8) & 0xFF) as u8;
        let lo = (feature_id & 0xFF) as u8;
        if let Ok(Some(resp)) = self.request(FEAT_IROOT as u8, 0, &[hi, lo, 0x00], 1000) {
            if !resp.is_empty() && resp[0] != 0 {
                return Some(resp[0]);
            }
        }
        None
    }

    fn set_control_reporting(&self, cid: u16, flags: u8) -> Result<()> {
        let feat = self.feat_idx.ok_or_else(|| anyhow!("REPROG_V4 feature missing"))?;
        let hi = ((cid >> 8) & 0xFF) as u8;
        let lo = (cid & 0xFF) as u8;
        let resp = self.request(feat, 3, &[hi, lo, flags, 0x00, 0x00], 1000)?;
        if resp.is_some() {
            Ok(())
        } else {
            Err(anyhow!("Failed to set control reporting"))
        }
    }

    fn divert_gesture_button(&mut self) -> Result<()> {
        // Try enabling RawXY report diversion first (0x33)
        if self.set_control_reporting(self.gesture_cid, 0x33).is_ok() {
            self.rawxy_enabled = true;
            log::info!("[HID++] Diverted gesture button with RawXY: OK");
        } else {
            // Fall back to standard diversion (0x03)
            self.set_control_reporting(self.gesture_cid, 0x03)?;
            self.rawxy_enabled = false;
            log::info!("[HID++] Diverted gesture button (standard): OK");
        }

        // Also attempt to divert Mode Shift (0x00C4) if available
        let _ = self.set_control_reporting(0x00C4, 0x03);

        Ok(())
    }

    fn undivert_gesture_button(&mut self) -> Result<()> {
        let flags = if self.rawxy_enabled { 0x22 } else { 0x02 }; // Restore default flags
        let _ = self.set_control_reporting(self.gesture_cid, flags);
        let _ = self.set_control_reporting(0x00C4, 0x02);
        self.rawxy_enabled = false;
        Ok(())
    }

    pub fn set_dpi(&self, dpi: u32) -> Result<()> {
        let idx = self.dpi_idx.ok_or_else(|| anyhow!("DPI adjustments not supported"))?;
        let hi = ((dpi >> 8) & 0xFF) as u8;
        let lo = (dpi & 0xFF) as u8;
        // setSensorDpi: function 3, params [sensorIdx=0, dpi_hi, dpi_lo]
        let resp = self.request(idx, 3, &[0x00, hi, lo], 1000)?;
        if resp.is_some() {
            log::info!("[HID++] DPI updated to {}", dpi);
            Ok(())
        } else {
            Err(anyhow!("Failed to set DPI"))
        }
    }

    pub fn read_dpi(&self) -> Result<u32> {
        let idx = self.dpi_idx.ok_or_else(|| anyhow!("DPI reading not supported"))?;
        // getSensorDpi: function 2, params [sensorIdx=0]
        let resp = self.request(idx, 2, &[0x00], 1000)?
            .ok_or_else(|| anyhow!("DPI read failed"))?;
        if resp.len() >= 3 {
            let actual = ((resp[1] as u32) << 8) | (resp[2] as u32);
            Ok(actual)
        } else {
            Err(anyhow!("Invalid DPI response size"))
        }
    }

    pub fn set_smart_shift(&self, mode: &str, enabled: bool, threshold: u8) -> Result<()> {
        let idx = self.smart_shift_idx.ok_or_else(|| anyhow!("Smart Shift not supported"))?;
        let write_fn = if self.smart_shift_enhanced { 2 } else { 1 };

        let params = if enabled {
            // Mode ratchet (0x02) + autoDisengage threshold
            [0x02, threshold, 0x00]
        } else if mode == "freespin" {
            // Mode freespin (0x01)
            [0x01, 0x00, 0x00]
        } else {
            // Fixed ratchet (SmartShift disabled), threshold=0xFF
            [0x02, 0xFF, 0x00]
        };

        let resp = self.request(idx, write_fn, &params, 1000)?;
        if resp.is_some() {
            log::info!("[HID++] Smart Shift configured: mode={}, enabled={}", mode, enabled);
            Ok(())
        } else {
            Err(anyhow!("Failed to configure Smart Shift"))
        }
    }

    pub fn get_layout_key(&self) -> String {
        let name = self.device_name.to_lowercase();
        // Return keyboard layout for MX Mechanical
        if name.contains("mechanical") || name.contains("mchncl") {
            return "mx_mechanical".to_string();
        }
        if name.contains("master 3s") {
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
        } else if name.contains("mx mechanical mini") {
            "mx_mechanical_mini".to_string()
        } else if name.contains("mx mechanical") {
            "mx_mechanical".to_string()
        } else {
            "generic".to_string()
        }
    }

    pub fn poll_events(&mut self) -> Result<Vec<HidppEvent>> {
        // Early-return if already closed — avoids repeated WARN spam between
        // disconnect detection and the engine's 2-second retain() cleanup.
        if self.device.is_none() {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();
        // Set timeout to 0 for a non-blocking read to avoid blocking the main loop
        match self.rx(0) {
            Err(e) => {
                // Log once on the actual disconnect, then close so next call hits the guard above.
                log::info!("[HID++] Device '{}' disconnected: {}. Will reconnect automatically.", self.device_name, e);
                self.close();
            }
            Ok(Some(raw)) => {
                if raw.len() < 4 {
                    return Ok(events);
                }
                let off = if raw[0] == SHORT_ID || raw[0] == LONG_ID { 1 } else { 0 };
                if off + 3 >= raw.len() {
                    return Ok(events);
                }

                let r_feat = raw[off + 1];
                let r_func = (raw[off + 2] >> 4) & 0x0F;
                let r_params = &raw[off + 3..];

                if Some(r_feat) == self.feat_idx {
                    if r_func == 1 {
                        // Gesture RawXY movement reports
                        if self.rawxy_enabled && self.held && r_params.len() >= 4 {
                            let dx = (((r_params[0] as u16) << 8) | (r_params[1] as u16)) as i16;
                            let dy = (((r_params[2] as u16) << 8) | (r_params[3] as u16)) as i16;
                            if dx != 0 || dy != 0 {
                                events.push(HidppEvent::GestureMove { dx, dy });
                            }
                        }
                    } else if r_func == 0 {
                        // Reprogrammable controls active state (pairs of CIDs terminated by 0x0000)
                        let mut active_cids = Vec::new();
                        let mut i = 0;
                        while i + 1 < r_params.len() {
                            let cid = ((r_params[i] as u16) << 8) | (r_params[i + 1] as u16);
                            if cid == 0 {
                                break;
                            }
                            active_cids.push(cid);
                            i += 2;
                        }

                        // Check gesture button CID
                        let gesture_now = active_cids.contains(&self.gesture_cid);
                        if gesture_now && !self.held {
                            self.held = true;
                            events.push(HidppEvent::GestureDown);
                        } else if !gesture_now && self.held {
                            self.held = false;
                            events.push(HidppEvent::GestureUp);
                        }

                        // Check mode shift CID (0x00C4)
                        let mode_shift_now = active_cids.contains(&0x00C4);
                        if mode_shift_now && !self.mode_shift_held {
                            self.mode_shift_held = true;
                            events.push(HidppEvent::ModeShiftDown);
                        } else if !mode_shift_now && self.mode_shift_held {
                            self.mode_shift_held = false;
                            events.push(HidppEvent::ModeShiftUp);
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(events)
    }
}
