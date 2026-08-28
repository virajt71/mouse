use anyhow::{anyhow, Result};
use std::time::{Duration, Instant};

fn tx_raw(
    device: &hidapi::HidDevice,
    dev_idx: u8,
    feat: u8,
    func: u8,
    params: &[u8],
) -> Result<()> {
    let mut buf = [0u8; 21]; // LONG_LEN + 1
    buf[1] = 0x11; // LONG_ID (0x11)
    buf[2] = dev_idx;
    buf[3] = feat;
    buf[4] = ((func & 0x0F) << 4) | (0x0A & 0x0F); // MY_SW is 0x0A

    for (i, &val) in params.iter().enumerate() {
        if i + 5 < buf.len() {
            buf[i + 5] = val;
        }
    }

    device.write(&buf[1..])?;
    Ok(())
}

fn rx_raw(device: &hidapi::HidDevice, timeout_ms: i32) -> Result<Option<Vec<u8>>> {
    let mut buf = [0u8; 64];
    let bytes_read = device.read_timeout(&mut buf, timeout_ms)?;
    if bytes_read > 0 {
        Ok(Some(buf[..bytes_read].to_vec()))
    } else {
        Ok(None)
    }
}

fn request_raw(
    device: &hidapi::HidDevice,
    dev_idx: u8,
    feat: u8,
    func: u8,
    params: &[u8],
    timeout_ms: u64,
) -> Result<Option<Vec<u8>>> {
    tx_raw(device, dev_idx, feat, func, params)?;
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    let expected_func = ((func & 0x0F) << 4) | (0x0A & 0x0F);

    while Instant::now() < deadline {
        let timeout = deadline
            .saturating_duration_since(Instant::now())
            .as_millis() as i32;
        if let Some(raw) = rx_raw(device, timeout.min(100))? {
            if raw.len() < 4 {
                continue;
            }
            let off = if raw[0] == 0x10 || raw[0] == 0x11 {
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
                return Ok(None);
            }

            if r_feat == feat && r_fsw == expected_func {
                return Ok(Some(r_params.to_vec()));
            }
        }
    }
    Ok(None)
}

fn find_feature_on_device(device: &hidapi::HidDevice, dev_idx: u8, feature_id: u16) -> Option<u8> {
    let hi = ((feature_id >> 8) & 0xFF) as u8;
    let lo = (feature_id & 0xFF) as u8;
    let params = [hi, lo, 0x00];

    // ROOT feature is 0x00, getFeature function is 0
    match request_raw(device, dev_idx, 0x00, 0x00, &params, 300) {
        Ok(Some(resp)) => {
            if !resp.is_empty() && resp[0] != 0 {
                Some(resp[0])
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn change_host(device: &hidapi::HidDevice, target_channel: u8) -> Result<()> {
    // 1. Probe for the correct dev_idx and locate CHANGE_HOST (0x1814) feature index dynamically
    let mut dev_idx_opt = None;
    let mut change_host_idx_opt = None;

    for idx in [0xFF, 1, 2, 3, 4, 5, 6] {
        if let Some(ch_idx) = find_feature_on_device(device, idx, 0x1814) {
            dev_idx_opt = Some(idx);
            change_host_idx_opt = Some(ch_idx);
            break;
        }
    }

    let dev_idx = dev_idx_opt.ok_or_else(|| anyhow!("Failed to locate active dev_idx"))?;
    let change_host_idx = change_host_idx_opt
        .ok_or_else(|| anyhow!("CHANGE_HOST feature (0x1814) not found on device"))?;

    log::info!(
        "[HID++ Channel Switch] Found CHANGE_HOST feature at index 0x{:02X} for dev_idx 0x{:02X}",
        change_host_idx,
        dev_idx
    );

    // 2. Call setCurrentHost (function index 1)
    log::info!("[HID++ Channel Switch] Setting host channel via function 1...");
    match request_raw(device, dev_idx, change_host_idx, 1, &[target_channel], 1000) {
        Ok(Some(_)) => {
            log::info!("[HID++ Channel Switch] Function 1 succeeded.");
        }
        other => {
            return Err(anyhow!(
                "Function 1 (setCurrentHost) failed or returned None: {:?}",
                other
            ));
        }
    }

    // 3. Verify with a `getCurrentHost` (function 0) read-back to confirm device actually switched
    log::info!("[HID++ Channel Switch] Verifying channel switch with read-back...");
    match request_raw(device, dev_idx, change_host_idx, 0, &[], 1000) {
        Ok(Some(resp)) => {
            if !resp.is_empty() {
                let active_host = if resp.len() >= 2 && resp[1] > 2 {
                    resp[0]
                } else if resp.len() >= 2 {
                    resp[1]
                } else {
                    resp[0]
                };
                if active_host != target_channel {
                    log::error!(
                        "[HID++ Channel Switch] Mismatch! Expected channel {}, but device reports channel {}",
                        target_channel,
                        active_host
                    );
                } else {
                    log::info!(
                        "[HID++ Channel Switch] Verification succeeded. Device is on channel {}.",
                        active_host
                    );
                }
            } else {
                log::error!("[HID++ Channel Switch] Empty response on read-back verification.");
            }
        }
        Err(e) => {
            log::error!(
                "[HID++ Channel Switch] Failed to read back host channel: {}",
                e
            );
        }
        Ok(None) => {
            log::info!("[HID++ Channel Switch] Read back verification returned None (device probably disconnected to switch).");
        }
    }

    Ok(())
}
