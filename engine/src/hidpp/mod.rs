pub mod protocol;
pub mod device;
pub mod diversion;

use anyhow::{anyhow, Result};
use hidapi::HidDevice;

use self::protocol::{SHORT_ID, LONG_ID};

pub enum HidppEvent {
    GestureDown,
    GestureUp,
    GestureMove { dx: i16, dy: i16 },
    ModeShiftDown,
    ModeShiftUp,
}

pub struct HidppClient {
    pub(crate) device: Option<HidDevice>,
    pub(crate) dev_idx: u8,
    pub(crate) feat_idx: Option<u8>,
    pub(crate) dpi_idx: Option<u8>,
    pub(crate) smart_shift_idx: Option<u8>,
    pub(crate) smart_shift_enhanced: bool,
    pub(crate) gesture_cid: u16,
    pub(crate) rawxy_enabled: bool,
    pub(crate) held: bool,
    pub(crate) mode_shift_held: bool,
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
            gesture_cid: 0x00C3,
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

    pub fn close(&mut self) {
        if self.device.is_some() {
            let _ = self.undivert_gesture_button();
        }
        self.device = None;
        self.device_name = "None".to_string();
        self.device_path = "".to_string();
        self.feat_idx = None;
        self.dpi_idx = None;
        self.smart_shift_idx = None;
    }

    pub fn set_dpi(&self, dpi: u32) -> Result<()> {
        let idx = self.dpi_idx.ok_or_else(|| anyhow!("DPI adjustments not supported"))?;
        let hi = ((dpi >> 8) & 0xFF) as u8;
        let lo = (dpi & 0xFF) as u8;
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
            [0x02, threshold, 0x00]
        } else if mode == "freespin" {
            [0x01, 0x00, 0x00]
        } else {
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

    pub fn poll_events(&mut self) -> Result<Vec<HidppEvent>> {
        if self.device.is_none() {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();
        match self.rx(0) {
            Err(e) => {
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
                        if self.rawxy_enabled && self.held && r_params.len() >= 4 {
                            let dx = (((r_params[0] as u16) << 8) | (r_params[1] as u16)) as i16;
                            let dy = (((r_params[2] as u16) << 8) | (r_params[3] as u16)) as i16;
                            if dx != 0 || dy != 0 {
                                events.push(HidppEvent::GestureMove { dx, dy });
                            }
                        }
                    } else if r_func == 0 {
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

                        let gesture_now = active_cids.contains(&self.gesture_cid);
                        if gesture_now && !self.held {
                            self.held = true;
                            events.push(HidppEvent::GestureDown);
                        } else if !gesture_now && self.held {
                            self.held = false;
                            events.push(HidppEvent::GestureUp);
                        }

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
