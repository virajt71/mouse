use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

pub static SWAY_SUPPORTED: AtomicBool = AtomicBool::new(true);

pub fn get_active_app_pid_sway() -> Option<u32> {
    if !SWAY_SUPPORTED.load(Ordering::Relaxed) {
        return None;
    }
    let output = Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .output()
        .ok();

    match output {
        Some(out) if out.status.success() => {
            let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
            find_focused_pid_sway(&json)
        }
        Some(_) => None,
        None => {
            log::info!("[AppDetector] swaymsg not found. Disabling Sway fallback.");
            SWAY_SUPPORTED.store(false, Ordering::Relaxed);
            None
        }
    }
}

pub fn find_focused_pid_sway(node: &serde_json::Value) -> Option<u32> {
    if node
        .get("focused")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        if let Some(pid) = node.get("pid").and_then(|v| v.as_u64()) {
            return Some(pid as u32);
        }
    }
    if let Some(nodes) = node.get("nodes").and_then(|v| v.as_array()) {
        for child in nodes {
            if let Some(pid) = find_focused_pid_sway(child) {
                return Some(pid);
            }
        }
    }
    if let Some(nodes) = node.get("floating_nodes").and_then(|v| v.as_array()) {
        for child in nodes {
            if let Some(pid) = find_focused_pid_sway(child) {
                return Some(pid);
            }
        }
    }
    None
}
