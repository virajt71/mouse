use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

pub static HYPRLAND_SUPPORTED: AtomicBool = AtomicBool::new(true);

pub fn get_active_app_pid_hyprland() -> Option<u32> {
    if !HYPRLAND_SUPPORTED.load(Ordering::Relaxed) {
        return None;
    }
    let output = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .ok();

    match output {
        Some(out) if out.status.success() => {
            let json: serde_json::Value =
                serde_json::from_slice(&out.stdout).ok()?;
            json.get("pid")?.as_u64().map(|p| p as u32)
        }
        Some(_) => None,
        None => {
            log::info!("[AppDetector] hyprctl not found. Disabling Hyprland fallback.");
            HYPRLAND_SUPPORTED.store(false, Ordering::Relaxed);
            None
        }
    }
}
