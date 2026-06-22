use super::sway::{find_focused_pid_sway, query_sway_i3_socket};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

pub static I3_MSG_SUPPORTED: AtomicBool = AtomicBool::new(true);

/// i3 IPC message type for GET_TREE = 1
const I3_MSG_GET_TREE: u32 = 1;

pub fn get_active_app_pid_i3() -> Option<u32> {
    if !I3_MSG_SUPPORTED.load(Ordering::Relaxed) {
        return None;
    }

    // Prefer direct socket communication to avoid spawning a subprocess
    if let Ok(socket_path) = std::env::var("I3SOCK") {
        if let Some(json) = query_sway_i3_socket(&socket_path, I3_MSG_GET_TREE) {
            return find_focused_pid_sway(&json);
        }
    }

    // Fallback: invoke i3-msg subprocess
    let output = Command::new("i3-msg")
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
            log::info!("[AppDetector] i3-msg not found. Disabling i3 fallback.");
            I3_MSG_SUPPORTED.store(false, Ordering::Relaxed);
            None
        }
    }
}
