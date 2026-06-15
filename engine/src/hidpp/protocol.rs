use super::HidppClient;
use anyhow::{anyhow, Result};
use std::time::{Duration, Instant};

pub const LOGI_VID: u16 = 0x046D;
pub const SHORT_ID: u8 = 0x10;
pub const LONG_ID: u8 = 0x11;
pub const LONG_LEN: usize = 20;
pub const MY_SW: u8 = 0x0A;

pub const FEAT_IROOT: u16 = 0x0000;
pub const FEAT_REPROG_V4: u16 = 0x1B04;
pub const FEAT_ADJ_DPI: u16 = 0x2201;
pub const FEAT_SMART_SHIFT: u16 = 0x2110;
pub const FEAT_SMART_SHIFT_ENHANCED: u16 = 0x2111;
pub const FEAT_CHANGE_HOST: u16 = 0x1814;

impl HidppClient {
    pub fn tx(&self, feat: u8, func: u8, params: &[u8]) -> Result<()> {
        let dev = self
            .device
            .as_ref()
            .ok_or_else(|| anyhow!("Device closed"))?;
        let mut buf = [0u8; LONG_LEN + 1];
        buf[1] = LONG_ID;
        buf[2] = self.dev_idx;
        buf[3] = feat;
        buf[4] = ((func & 0x0F) << 4) | (MY_SW & 0x0F);

        for (i, &val) in params.iter().enumerate() {
            if i + 5 < buf.len() {
                buf[i + 5] = val;
            }
        }

        dev.write(&buf[1..])?;
        Ok(())
    }

    pub fn rx(&self, timeout_ms: i32) -> Result<Option<Vec<u8>>> {
        let dev = self
            .device
            .as_ref()
            .ok_or_else(|| anyhow!("Device closed"))?;
        let mut buf = [0u8; 64];
        let bytes_read = dev.read_timeout(&mut buf, timeout_ms)?;
        if bytes_read > 0 {
            Ok(Some(buf[..bytes_read].to_vec()))
        } else {
            Ok(None)
        }
    }

    pub fn request(
        &self,
        feat: u8,
        func: u8,
        params: &[u8],
        timeout_ms: u64,
    ) -> Result<Option<Vec<u8>>> {
        self.tx(feat, func, params)?;
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        let expected_func = ((func & 0x0F) << 4) | (MY_SW & 0x0F);

        while Instant::now() < deadline {
            let timeout = deadline
                .saturating_duration_since(Instant::now())
                .as_millis() as i32;
            if let Some(raw) = self.rx(timeout.min(100))? {
                if raw.len() < 4 {
                    continue;
                }
                let off = if raw[0] == SHORT_ID || raw[0] == LONG_ID {
                    1
                } else {
                    0
                };
                if off + 3 >= raw.len() {
                    continue;
                }

                let r_feat = raw[off + 1];
                let r_fsw = raw[off + 2];
                let r_params = &raw[off + 3..];

                if r_feat == 0xFF {
                    let err_code = if r_params.len() > 1 { r_params[1] } else { 0 };
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

    pub fn find_feature(&self, feature_id: u16) -> Option<u8> {
        let hi = ((feature_id >> 8) & 0xFF) as u8;
        let lo = (feature_id & 0xFF) as u8;
        if let Ok(Some(resp)) = self.request(FEAT_IROOT as u8, 0, &[hi, lo, 0x00], 300) {
            if !resp.is_empty() && resp[0] != 0 {
                return Some(resp[0]);
            }
        }
        None
    }
}
