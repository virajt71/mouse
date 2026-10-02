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
pub const FEAT_BACKLIGHT2: u16 = 0x1982;
pub const FEAT_UNIFIED_BATTERY: u16 = 0x1004;
pub const FEAT_BATTERY_STATUS: u16 = 0x1000;
pub const FEAT_BATTERY_VOLTAGE: u16 = 0x1001;

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

/// Approximate level label for the unified battery feature's level byte
/// (0x1004 `getBatteryInfo` payload[1]). Matches the vendored `hidpp`
/// crate's [`BatteryLevel`] discriminant values:
/// Critical = 1, Low = 2, Good = 4, Full = 8.
pub fn unified_battery_level_label(level: u8) -> String {
    match level {
        1 => "critical".into(),
        2 => "low".into(),
        4 => "good".into(),
        8 => "full".into(),
        _ => "unknown_level".into(),
    }
}

/// Full unified-battery decode: percentage, level label, status label.
/// Mirrors OpenLogi's decode (0x1004 `getBatteryInfo` → payload[percentage,
/// level, status]) plus the level label the existing
/// `parse_unified_battery` ignored.
pub fn parse_unified_battery_full(payload: &[u8]) -> Option<(u8, String, String)> {
    if payload.len() < 3 {
        return None;
    }
    let percentage = payload[0].min(100);
    let level = unified_battery_level_label(payload[1]);
    let status = unified_battery_status_label(payload[2]);
    Some((percentage, level, status))
}
/// Decode a unified-battery (0x1004) response &[percentage, level, status].
/// Mirrors OpenLogi's `decode` for this feature.
pub fn parse_unified_battery(payload: &[u8]) -> (u8, String) {
    let percentage = payload[0].min(100);
    let status = unified_battery_status_label(payload.get(2).copied().unwrap_or(0));
    (percentage, status)
}

/// Charge-status label for the unified battery feature's status byte.
pub fn unified_battery_status_label(status: u8) -> String {
    match status {
        0 => "discharging".into(),
        1 => "charging".into(),
        2 => "charging_nearly_full".into(),
        3 => "full".into(),
        4 => "charging_slow".into(),
        5 => "invalid_battery".into(),
        6 => "thermal_error".into(),
        7 => "charging_error".into(),
        _ => "unknown".into(),
    }
}


/// Status label for the 0x1001 voltage-battery status byte.
/// Values mirror the HID++ 2.0 `batteryVoltageStatus` usage (OpenLogi's
/// `BatteryVoltageStatus` enum): discharging, charging, fast charge, full,
/// slow charge, invalid, thermal error, charging error.
pub fn voltage_battery_status_label(status: u8) -> &'static str {
    match status {
        0 => "discharging",
        1 => "charging",
        2 => "charging_fast",
        3 => "full",
        4 => "charging_slow",
        5 => "invalid_battery",
        6 => "thermal_error",
        7 => "charging_error",
        _ => "unknown",
    }
}

/// Estimate a charge percentage from a 0x1001 voltage reading (millivolts).
///
/// The 0x1001 feature reports battery voltage, not percent, so we model the
/// discharge curve ourselves. This table is a conservative Solaar-style
/// mapping for single-cell Li-Po packs; when in doubt we round down rather
/// than over-promise charge. Calibrate against a real device before trusting
/// the exact numbers.
pub fn voltage_to_battery_percent(mv: u16) -> u8 {
    const CURVE: &[(u16, u8)] = &[
        (4200, 100),
        (4100, 90),
        (4000, 80),
        (3900, 70),
        (3800, 60),
        (3700, 50),
        (3600, 40),
        (3500, 30),
        (3400, 20),
        (3300, 10),
        (3200, 5),
        (3100, 2),
        (3000, 0),
    ];

    if mv >= CURVE[0].0 {
        return CURVE[0].1;
    }
    if mv <= CURVE[CURVE.len() - 1].0 {
        return CURVE[CURVE.len() - 1].1;
    }
    for i in 1..CURVE.len() {
        if mv >= CURVE[i].0 {
            let (mv_high, pct_high) = CURVE[i - 1];
            let (mv_low, pct_low) = CURVE[i];
            let span_mv = (mv_high - mv_low) as u16;
            let span_pct = (pct_high - pct_low) as u16;
            // integer-only interpolation, rounding to nearest
            let frac = ((mv - mv_low) as u16 * 100 + span_mv / 2) / span_mv;
            return (pct_low as u16 + (span_pct * frac / 100)) as u8;
        }
    }
    0
}

/// Decode a 0x1001 `getBatteryInfo` response into (millivolts, status_label).
///
/// Conservative format assumption: [voltage_msb, voltage_lsb, status_byte].
/// This matches common HID++ 0x1001 implementations; verify against a real
/// G-series device before trusting the exact byte order (log raw bytes at
/// debug level in the caller if unsure).
pub fn parse_voltage_battery(payload: &[u8]) -> Option<(u16, &'static str)> {
    if payload.len() < 2 {
        return None;
    }
    let mv = ((payload[0] as u16) << 8) | payload[1] as u16;
    let status = if payload.len() >= 3 {
        voltage_battery_status_label(payload[2])
    } else {
        "unknown"
    };
    Some((mv, status))
}

/// Charge-status label for the legacy battery feature's (0x1000) status byte.
pub fn legacy_status_label(status: u8) -> String {
    match status {
        0 => "discharging".into(),
        1 => "recharging".into(),
        2 => "almost_full".into(),
        3 => "full".into(),
        4 => "slow_recharge".into(),
        5 => "invalid_battery".into(),
        6 => "thermal_error".into(),
        7 => "other".into(),
        _ => "unknown".into(),
    }
}
