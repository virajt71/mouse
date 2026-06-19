pub mod device;
pub mod diversion;
pub mod protocol;

use anyhow::{anyhow, Result};
use hidapi::HidDevice;

use self::protocol::{LONG_ID, MY_SW, SHORT_ID};

pub enum HidppEvent {
    GestureDown,
    GestureUp,
    GestureMove { dx: i16, dy: i16 },
    ModeShiftDown,
    ModeShiftUp,
    BacklightChanged { enabled: bool, effect_id: u8 },
}

pub struct HidppClient {
    pub(crate) device: Option<HidDevice>,
    pub(crate) dev_idx: u8,
    pub(crate) feat_idx: Option<u8>,
    pub(crate) dpi_idx: Option<u8>,
    pub(crate) smart_shift_idx: Option<u8>,
    pub(crate) smart_shift_enhanced: bool,
    pub(crate) change_host_idx: Option<u8>,
    pub(crate) backlight_feat_idx: Option<u8>,
    pub(crate) gesture_cid: u16,
    pub layout_from_pid: Option<&'static str>,
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

#[derive(Debug, Clone)]
pub struct BacklightState {
    pub enabled: u8,
    pub options: u8,
    pub supported: u8,
    pub effects: u16,
    pub level: u8,
    pub dho: u16,
    pub dhi: u16,
    pub dpow: u16,
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
            change_host_idx: None,
            backlight_feat_idx: None,
            gesture_cid: 0x00C3,
            layout_from_pid: None,
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
        self.change_host_idx = None;
        self.backlight_feat_idx = None;
        self.layout_from_pid = None;
    }

    pub fn set_dpi(&self, dpi: u32) -> Result<()> {
        let idx = self
            .dpi_idx
            .ok_or_else(|| anyhow!("DPI adjustments not supported"))?;
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
        let idx = self
            .dpi_idx
            .ok_or_else(|| anyhow!("DPI reading not supported"))?;
        let resp = self
            .request(idx, 2, &[0x00], 1000)?
            .ok_or_else(|| anyhow!("DPI read failed"))?;
        if resp.len() >= 3 {
            let actual = ((resp[1] as u32) << 8) | (resp[2] as u32);
            Ok(actual)
        } else {
            Err(anyhow!("Invalid DPI response size"))
        }
    }

    pub fn set_smart_shift(&self, mode: &str, enabled: bool, threshold: u8) -> Result<()> {
        let idx = self
            .smart_shift_idx
            .ok_or_else(|| anyhow!("Smart Shift not supported"))?;
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
            log::info!(
                "[HID++] Smart Shift configured: mode={}, enabled={}",
                mode,
                enabled
            );
            Ok(())
        } else {
            Err(anyhow!("Failed to configure Smart Shift"))
        }
    }

    pub fn read_backlight_state(&self, feat_idx: u8) -> Result<BacklightState> {
        let resp = self
            .request(feat_idx, 0x00, &[], 1000)?
            .ok_or_else(|| anyhow!("No reply from device for backlight state read"))?;

        if resp.len() < 12 {
            return Err(anyhow!(
                "Backlight state response too short: expected at least 12 bytes, got {}",
                resp.len()
            ));
        }

        let enabled = resp[0];
        let options = resp[1];
        let supported = resp[2];
        let effects = u16::from_le_bytes([resp[3], resp[4]]);
        let level = resp[5];
        let dho = u16::from_le_bytes([resp[6], resp[7]]);
        let dhi = u16::from_le_bytes([resp[8], resp[9]]);
        let dpow = u16::from_le_bytes([resp[10], resp[11]]);

        Ok(BacklightState {
            enabled,
            options,
            supported,
            effects,
            level,
            dho,
            dhi,
            dpow,
        })
    }

    pub fn write_backlight_state(
        &self,
        feat_idx: u8,
        state: &BacklightState,
        effect_id: u8,
    ) -> Result<()> {
        let mut params = [0u8; 10];
        params[0] = state.enabled;
        params[1] = state.options;
        params[2] = effect_id;
        params[3] = state.level;

        let dho_bytes = state.dho.to_le_bytes();
        params[4] = dho_bytes[0];
        params[5] = dho_bytes[1];

        let dhi_bytes = state.dhi.to_le_bytes();
        params[6] = dhi_bytes[0];
        params[7] = dhi_bytes[1];

        let dpow_bytes = state.dpow.to_le_bytes();
        params[8] = dpow_bytes[0];
        params[9] = dpow_bytes[1];

        let resp = self.request(feat_idx, 0x10, &params, 1000)?;
        if resp.is_some() {
            Ok(())
        } else {
            Err(anyhow!("Failed to write backlight state (0x1982)"))
        }
    }

    pub fn set_backlight_effect(&self, effect: &str) -> Result<()> {
        log::info!(
            "[HID++] Attempting to set backlight effect on '{}' to {}",
            self.device_name,
            effect
        );

        if let Some(feat_idx) = self.find_feature(0x1982) {
            let mut state = self.read_backlight_state(feat_idx)?;
            let effect_id = match effect {
                "Static" => 0x01,
                "Contrast" => 0x02,
                "Breathing" => 0x03,
                "Waves" => 0x04,
                "Reaction" => 0x05,
                "Random" => 0x06,
                _ => 0x01,
            };
            state.enabled = 1;
            state.options = (state.options & 0x07) | (0x03 << 3); // Force manual mode for level
            self.write_backlight_state(feat_idx, &state, effect_id)?;
            log::info!(
                "[HID++] Backlight effect '{}' successfully sent to device (0x1982)",
                effect
            );
            Ok(())
        } else if let Some(feat_idx) = self.find_feature(0x8070) {
            let effect_id = match effect {
                "Static" => 0x01,
                "Contrast" => 0x02,
                "Breathing" => 0x03,
                "Waves" => 0x04,
                "Reaction" => 0x05,
                "Random" => 0x06,
                _ => 0x01,
            };
            let params = [effect_id, 0x00, 0x00, 0x00, 0x00];
            let resp = self.request(feat_idx, 3, &params, 1000)?;
            if resp.is_some() {
                log::info!(
                    "[HID++] Backlight effect '{}' successfully sent to device (0x8070)",
                    effect
                );
                Ok(())
            } else {
                Err(anyhow!(
                    "Failed to apply backlight effect to device (0x8070)"
                ))
            }
        } else {
            Err(anyhow!(
                "No backlight control feature (0x1982 or 0x8070) found on this device"
            ))
        }
    }

    pub fn set_backlight_enabled(&self, enabled: bool) -> Result<()> {
        log::info!(
            "[HID++] Attempting to set backlight enabled to {} on '{}'",
            enabled,
            self.device_name
        );

        if let Some(feat_idx) = self.find_feature(0x1982) {
            let mut state = self.read_backlight_state(feat_idx)?;
            state.enabled = if enabled { 1 } else { 0 };
            if enabled {
                state.options = (state.options & 0x07) | (0x03 << 3); // Force manual mode for level
            }
            self.write_backlight_state(feat_idx, &state, 0xFF)?; // 0xFF keeps current effect
            log::info!(
                "[HID++] Backlight enabled state {} successfully sent to device (0x1982)",
                enabled
            );
            Ok(())
        } else if let Some(feat_idx) = self.find_feature(0x8070) {
            let params = [enabled as u8, 0x00, 0x00];
            let resp = self.request(feat_idx, 1, &params, 1000)?;
            if resp.is_some() {
                log::info!(
                    "[HID++] Backlight enabled state {} successfully sent to device (0x8070)",
                    enabled
                );
                Ok(())
            } else {
                Err(anyhow!(
                    "Failed to apply backlight state to device (0x8070)"
                ))
            }
        } else {
            Err(anyhow!(
                "No backlight control feature (0x1982 or 0x8070) found on this device"
            ))
        }
    }

    pub fn poll_events(&mut self) -> Result<Vec<HidppEvent>> {
        if self.device.is_none() {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();
        match self.rx(16) {
            Err(e) => {
                log::info!(
                    "[HID++] Device '{}' disconnected: {}. Will reconnect automatically.",
                    self.device_name,
                    e
                );
                self.close();
            }
            Ok(Some(raw)) => {
                if raw.len() < 4 {
                    return Ok(events);
                }
                let off = if raw[0] == SHORT_ID || raw[0] == LONG_ID {
                    1
                } else {
                    0
                };
                if off + 3 >= raw.len() {
                    return Ok(events);
                }

                let r_feat = raw[off + 1];
                let r_fsw = raw[off + 2];
                let r_func = (r_fsw >> 4) & 0x0F;
                let r_params = &raw[off + 3..];

                // HID++ 0x1982 backlight state-change notification
                if Some(r_feat) == self.backlight_feat_idx && r_params.len() >= 2 {
                    log::debug!(
                        "[HID++] BL raw: feat=0x{:02X} r_func=0x{:02X} sw_nibble=0x{:02X} params={:02X?}",
                        r_feat, r_func, r_fsw & 0x0F, r_params
                    );
                    // Unsolicited push has SW nibble 0x00; skip our own request echoes
                    let sw_nibble = r_fsw & 0x0F;
                    if sw_nibble != MY_SW {
                        let enabled = r_params[0] != 0;
                        // Based on HID++ 2.0 specifications for feature 0x1982:
                        // Byte 0: enabled, Byte 2: effect_id
                        let effect_id = r_params.get(2).copied().unwrap_or(0);
                        events.push(HidppEvent::BacklightChanged { enabled, effect_id });
                    }
                }

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
pub mod tests;
