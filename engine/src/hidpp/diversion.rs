use anyhow::{anyhow, Result};
use super::HidppClient;

impl HidppClient {
    pub fn set_control_reporting(&self, cid: u16, flags: u8) -> Result<()> {
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

    pub fn divert_gesture_button(&mut self) -> Result<()> {
        if self.set_control_reporting(self.gesture_cid, 0x33).is_ok() {
            self.rawxy_enabled = true;
            log::info!("[HID++] Diverted gesture button with RawXY: OK");
        } else {
            self.set_control_reporting(self.gesture_cid, 0x03)?;
            self.rawxy_enabled = false;
            log::info!("[HID++] Diverted gesture button (standard): OK");
        }

        let _ = self.set_control_reporting(0x00C4, 0x03);
        Ok(())
    }

    pub fn undivert_gesture_button(&mut self) -> Result<()> {
        let flags = if self.rawxy_enabled { 0x22 } else { 0x02 };
        let _ = self.set_control_reporting(self.gesture_cid, flags);
        let _ = self.set_control_reporting(0x00C4, 0x02);
        self.rawxy_enabled = false;
        Ok(())
    }
}
