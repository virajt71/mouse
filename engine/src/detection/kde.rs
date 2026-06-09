use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

pub static KDOTOOL_SUPPORTED: AtomicBool = AtomicBool::new(true);

pub fn get_pid_from_kdotool() -> Option<u32> {
    if !KDOTOOL_SUPPORTED.load(Ordering::Relaxed) {
        return None;
    }

    let output = Command::new("kdotool")
        .args(["getactivewindow", "getwindowpid"])
        .output()
        .ok();

    match output {
        Some(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Ok(pid) = stdout.trim().parse::<u32>() {
                    return Some(pid);
                }
            }
            None
        }
        None => {
            log::info!("[AppDetector] kdotool command not found. Disabling fallback.");
            KDOTOOL_SUPPORTED.store(false, Ordering::Relaxed);
            None
        }
    }
}
