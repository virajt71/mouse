use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};

pub struct AppDetector {
    on_change: Arc<dyn Fn(String) + Send + Sync + 'static>,
    interval: Duration,
    running: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

// ---------------------------------------------------------------------------
// X11 / XWayland — stacking-order approach
//
// _NET_ACTIVE_WINDOW is unreliable under GNOME 46 Wayland (always 0x0).
// _NET_CLIENT_LIST_STACKING is maintained by Mutter for XWayland compatibility;
// the LAST entry in the list is the topmost (focused) window.
// This works for any app that runs via XWayland (the majority on Ubuntu 24.04).
// ---------------------------------------------------------------------------
fn get_active_app_pid_x11() -> Option<u32> {
    let display = std::env::var("DISPLAY").ok()?;
    if display.is_empty() {
        return None;
    }

    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Try _NET_ACTIVE_WINDOW first (works on pure X11 and some compositors)
    if let Some(pid) = get_pid_via_active_window(&conn, root) {
        return Some(pid);
    }

    // Fallback: read _NET_CLIENT_LIST_STACKING and take the topmost window.
    // Under GNOME/Mutter on Wayland, _NET_ACTIVE_WINDOW is 0x0 but the stacking
    // list is up-to-date. The last entry is the focused window.
    get_pid_via_stacking_list(&conn, root)
}

fn get_pid_via_active_window(conn: &impl ConnectionExt, root: u32) -> Option<u32> {
    let active_window_atom = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .ok()?
        .reply()
        .ok()?
        .atom;
    let pid_atom = conn
        .intern_atom(false, b"_NET_WM_PID")
        .ok()?
        .reply()
        .ok()?
        .atom;

    let resp = conn
        .get_property(false, root, active_window_atom, AtomEnum::WINDOW, 0, 1)
        .ok()?
        .reply()
        .ok()?;

    if resp.value_len == 0 {
        return None;
    }

    let window_id = resp.value32()?.next()?;
    // A window ID of 0 means "no active window" — skip.
    if window_id == 0 {
        return None;
    }

    let pid_resp = conn
        .get_property(false, window_id, pid_atom, AtomEnum::CARDINAL, 0, 1)
        .ok()?
        .reply()
        .ok()?;

    if pid_resp.value_len == 0 {
        return None;
    }

    let pid = pid_resp.value32()?.next();
    pid
}

fn get_pid_via_stacking_list(conn: &impl ConnectionExt, root: u32) -> Option<u32> {
    let stacking_atom = conn
        .intern_atom(false, b"_NET_CLIENT_LIST_STACKING")
        .ok()?
        .reply()
        .ok()?
        .atom;
    let pid_atom = conn
        .intern_atom(false, b"_NET_WM_PID")
        .ok()?
        .reply()
        .ok()?
        .atom;

    // Fetch up to 1024 window IDs (4 bytes each → 4096 bytes)
    let resp = conn
        .get_property(false, root, stacking_atom, AtomEnum::WINDOW, 0, 1024)
        .ok()?
        .reply()
        .ok()?;

    if resp.value_len == 0 {
        return None;
    }

    // The last entry in the stacking list is the topmost (focused) window.
    let window_id = resp.value32()?.last()?;
    if window_id == 0 {
        return None;
    }

    let pid_resp = conn
        .get_property(false, window_id, pid_atom, AtomEnum::CARDINAL, 0, 1)
        .ok()?
        .reply()
        .ok()?;

    if pid_resp.value_len == 0 {
        return None;
    }

    let pid = pid_resp.value32()?.next();
    pid
}

// ---------------------------------------------------------------------------
// GNOME Shell D-Bus Eval — works for native Wayland apps (future-proof)
//
// Disabled by default in GNOME 45+ but included as a forward-compat fallback.
// Output format: "(true, '12345')\n"
// ---------------------------------------------------------------------------
fn get_active_app_pid_gnome_shell() -> Option<u32> {
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
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Expect "(true, '12345')" — extract the number between single quotes.
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

// ---------------------------------------------------------------------------
// xdotool / kdotool subprocess helpers
// ---------------------------------------------------------------------------
fn get_pid_from_cmd(cmd_name: &str) -> Option<u32> {
    let output = Command::new(cmd_name)
        .args(["getactivewindow", "getwindowpid"])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(pid) = stdout.trim().parse::<u32>() {
            return Some(pid);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Top-level dispatcher
//
// Priority:
//   1. x11rb (stacking list) — zero-subprocess, works for all XWayland apps
//   2. GNOME Shell D-Bus Eval — works for native Wayland apps (if enabled)
//   3. kdotool — KDE Wayland
//   4. xdotool — legacy X11 fallback
// ---------------------------------------------------------------------------
fn get_active_app_pid() -> Option<u32> {
    let has_display = std::env::var("DISPLAY")
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_uppercase();
    let is_gnome = desktop.contains("GNOME");
    let is_kde = desktop.contains("KDE");

    // 1. x11rb via XWayland — try whenever a DISPLAY is available.
    //    Works for all XWayland-backed apps regardless of XDG_SESSION_TYPE.
    if has_display {
        if let Some(pid) = get_active_app_pid_x11() {
            return Some(pid);
        }
    }

    // 2. GNOME Shell D-Bus Eval — catches native Wayland apps on GNOME.
    //    org.gnome.Shell.Eval is disabled by default in GNOME 45+ but included
    //    as a forward-compat path; it costs one failed subprocess call if disabled.
    if is_gnome {
        if let Some(pid) = get_active_app_pid_gnome_shell() {
            return Some(pid);
        }
    }

    // 3. kdotool — KDE Wayland only.
    if is_kde {
        if let Some(pid) = get_pid_from_cmd("kdotool") {
            return Some(pid);
        }
    }

    // 4. xdotool — legacy X11 / XWayland subprocess fallback.
    if has_display {
        if let Some(pid) = get_pid_from_cmd("xdotool") {
            return Some(pid);
        }
    }

    None
}

fn get_exe_for_pid(pid: u32) -> Option<String> {
    let path = format!("/proc/{}/exe", pid);
    if let Ok(target) = std::fs::read_link(path) {
        if let Some(filename) = target.file_name() {
            return Some(filename.to_string_lossy().to_string());
        }
    }
    None
}

pub fn get_foreground_exe() -> Option<String> {
    let pid = get_active_app_pid()?;
    get_exe_for_pid(pid)
}

impl AppDetector {
    pub fn new<F>(on_change: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        AppDetector {
            on_change: Arc::new(on_change),
            interval: Duration::from_millis(300),
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    pub fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let interval = self.interval;
        let on_change = self.on_change.clone();

        let handle = thread::Builder::new()
            .name("AppDetector".to_string())
            .spawn(move || {
                let mut last_exe = String::new();
                let mut tool_warned = false;

                while running.load(Ordering::SeqCst) {
                    match get_foreground_exe() {
                        Some(exe) => {
                            if exe != last_exe {
                                last_exe = exe.clone();
                                on_change(exe);
                            }
                        }
                        None => {
                            if !tool_warned {
                                log::warn!(
                                    "[AppDetector] Could not detect the active window. \
                                     Tried: x11rb (_NET_CLIENT_LIST_STACKING), \
                                     GNOME Shell D-Bus, kdotool, xdotool. \
                                     Per-app profile switching will be disabled."
                                );
                                tool_warned = true;
                            }
                        }
                    }
                    thread::sleep(interval);
                }
            })
            .expect("Failed to spawn AppDetector thread");

        self.thread_handle = Some(handle);
    }

    pub fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
