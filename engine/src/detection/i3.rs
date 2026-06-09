use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use super::sway::find_focused_pid_sway;

pub static I3_MSG_SUPPORTED: AtomicBool = AtomicBool::new(true);

pub fn get_active_app_pid_i3() -> Option<u32> {
    if !I3_MSG_SUPPORTED.load(Ordering::Relaxed) {
        return None;
    }
    let output = Command::new("i3-msg")
        .args(["-t", "get_tree"])
        .output()
        .ok();

    match output {
        Some(out) if out.status.success() => {
            let json: serde_json::Value =
                serde_json::from_slice(&out.stdout).ok()?;
            find_focused_pid_sway(&json)
        }
        Some(_) => None,
        None => {
            log::info!("[AppDetector] i3-msg not found. Disabling i3 fallback.");
            I3_MSG_SUPPORTED.store(false, Ordering::Relaxed);
            None
        }
    }
}
