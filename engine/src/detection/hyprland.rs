use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

pub static HYPRLAND_SUPPORTED: AtomicBool = AtomicBool::new(true);

/// Query the Hyprland IPC socket directly to get the active window JSON.
/// Sends "j/activewindow\n" and reads the response.
fn query_hyprland_socket() -> Option<serde_json::Value> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok()?;
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    let socket_path = format!("{}/hypr/{}/.socket.sock", runtime_dir, sig);

    let mut stream = UnixStream::connect(&socket_path).ok()?;
    // Hyprland IPC: send "j/activewindow" to get JSON output
    stream.write_all(b"j/activewindow\n").ok()?;

    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;

    serde_json::from_str(&response).ok()
}

pub fn get_active_app_pid_hyprland() -> Option<u32> {
    if !HYPRLAND_SUPPORTED.load(Ordering::Relaxed) {
        return None;
    }

    // Prefer direct socket IPC to avoid subprocess overhead
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        if let Some(json) = query_hyprland_socket() {
            return json.get("pid")?.as_u64().map(|p| p as u32);
        }
    }

    // Fallback: invoke hyprctl subprocess
    let output = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .ok();

    match output {
        Some(out) if out.status.success() => {
            let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
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
