use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

pub static GNOME_EVAL_SUPPORTED: AtomicBool = AtomicBool::new(true);

pub fn get_active_app_pid_gnome_shell() -> Option<u32> {
    if !GNOME_EVAL_SUPPORTED.load(Ordering::Relaxed) {
        return None;
    }
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/gnome/Shell",
            "--method",
            "org.gnome.Shell.Eval",
            "global.display.focus_window?.get_pid() ?? -1",
        ])
        .output()
        .ok();

    match output {
        Some(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let s = stdout.trim();
            if !s.starts_with("(true,") {
                return None;
            }
            let start = s.find('\'')?;
            let end = s.rfind('\'')?;
            if start >= end {
                return None;
            }
            let pid_str = &s[start + 1..end];
            let pid: i64 = pid_str.trim().parse().ok()?;
            if pid > 0 {
                Some(pid as u32)
            } else {
                None
            }
        }
        Some(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if stderr.contains("doesn't exist")
                || stderr.contains("unknown method")
                || stderr.contains("InvalidArgs")
            {
                log::info!("[AppDetector] GNOME Shell Eval method is not supported (likely disabled). Disabling GNOME Eval fallback.");
                GNOME_EVAL_SUPPORTED.store(false, Ordering::Relaxed);
            }
            None
        }
        None => {
            log::info!("[AppDetector] gdbus command not found. Disabling GNOME Eval fallback.");
            GNOME_EVAL_SUPPORTED.store(false, Ordering::Relaxed);
            None
        }
    }
}
