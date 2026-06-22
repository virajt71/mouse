use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

pub static SWAY_SUPPORTED: AtomicBool = AtomicBool::new(true);

pub fn query_sway_i3_socket(socket_path: &str, msg_type: u32) -> Option<serde_json::Value> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(socket_path).ok()?;
    let mut header = Vec::with_capacity(14);
    header.extend_from_slice(b"i3-ipc");
    header.extend_from_slice(&0u32.to_le_bytes());
    header.extend_from_slice(&msg_type.to_le_bytes());

    stream.write_all(&header).ok()?;

    let mut resp_magic = [0u8; 6];
    stream.read_exact(&mut resp_magic).ok()?;
    if &resp_magic != b"i3-ipc" {
        return None;
    }

    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).ok()?;
    let len = u32::from_le_bytes(len_bytes) as usize;

    let mut type_bytes = [0u8; 4];
    stream.read_exact(&mut type_bytes).ok()?;

    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).ok()?;

    serde_json::from_slice(&payload).ok()
}

pub fn get_active_app_pid_sway() -> Option<u32> {
    if !SWAY_SUPPORTED.load(Ordering::Relaxed) {
        return None;
    }

    if let Ok(socket_path) = std::env::var("SWAYSOCK") {
        if let Some(json) = query_sway_i3_socket(&socket_path, 4) {
            return find_focused_pid_sway(&json);
        }
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
