use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};
use x11rb::errors::ReplyError;

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
fn get_pid_via_active_window(conn: &impl Connection, root: u32) -> Result<Option<u32>, ReplyError> {
    let active_window_atom = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")?
        .reply()?
        .atom;
    let pid_atom = conn
        .intern_atom(false, b"_NET_WM_PID")?
        .reply()?
        .atom;

    let resp = conn
        .get_property(false, root, active_window_atom, AtomEnum::WINDOW, 0, 1)?
        .reply()?;

    if resp.value_len == 0 {
        return Ok(None);
    }

    let window_id = resp.value32().and_then(|mut iter| iter.next());
    let Some(window_id) = window_id else {
        return Ok(None);
    };
    // A window ID of 0 means "no active window" — skip.
    if window_id == 0 {
        return Ok(None);
    }

    let pid_resp = conn
        .get_property(false, window_id, pid_atom, AtomEnum::CARDINAL, 0, 1)?
        .reply()?;

    if pid_resp.value_len == 0 {
        return Ok(None);
    }

    let pid = pid_resp.value32().and_then(|mut iter| iter.next());
    Ok(pid)
}

fn get_pid_via_stacking_list(conn: &impl Connection, root: u32) -> Result<Option<u32>, ReplyError> {
    let stacking_atom = conn
        .intern_atom(false, b"_NET_CLIENT_LIST_STACKING")?
        .reply()?
        .atom;
    let pid_atom = conn
        .intern_atom(false, b"_NET_WM_PID")?
        .reply()?
        .atom;

    // Fetch up to 1024 window IDs (4 bytes each → 4096 bytes)
    let resp = conn
        .get_property(false, root, stacking_atom, AtomEnum::WINDOW, 0, 1024)?
        .reply()?;

    if resp.value_len == 0 {
        return Ok(None);
    }

    // The last entry in the stacking list is the topmost (focused) window.
    let window_id = resp.value32().and_then(|iter| iter.last());
    let Some(window_id) = window_id else {
        return Ok(None);
    };
    if window_id == 0 {
        return Ok(None);
    }

    let pid_resp = conn
        .get_property(false, window_id, pid_atom, AtomEnum::CARDINAL, 0, 1)?
        .reply()?;

    if pid_resp.value_len == 0 {
        return Ok(None);
    }

    let pid = pid_resp.value32().and_then(|mut iter| iter.next());
    Ok(pid)
}

fn get_active_app_pid_x11_persistent(
    conn: &impl Connection,
    screen_num: usize,
) -> Result<Option<u32>, ReplyError> {
    let setup = conn.setup();
    if screen_num >= setup.roots.len() {
        return Ok(None);
    }
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    if let Some(pid) = get_pid_via_active_window(conn, root)? {
        return Ok(Some(pid));
    }

    get_pid_via_stacking_list(conn, root)
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
// Fallbacks dispatcher
// ---------------------------------------------------------------------------
fn get_active_app_pid_fallbacks() -> Option<u32> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_uppercase();
    let is_gnome = desktop.contains("GNOME");
    let is_kde = desktop.contains("KDE");
    let has_display = std::env::var("DISPLAY")
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    // 1. GNOME Shell D-Bus Eval — catches native Wayland apps on GNOME.
    if is_gnome {
        if let Some(pid) = get_active_app_pid_gnome_shell() {
            return Some(pid);
        }
    }

    // 2. kdotool — KDE Wayland only.
    if is_kde {
        if let Some(pid) = get_pid_from_cmd("kdotool") {
            return Some(pid);
        }
    }

    // 3. xdotool — legacy X11 / XWayland subprocess fallback.
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
    if let Ok(display) = std::env::var("DISPLAY") {
        if !display.is_empty() {
            if let Ok((conn, screen_num)) = x11rb::connect(None) {
                if let Ok(Some(pid)) = get_active_app_pid_x11_persistent(&conn, screen_num) {
                    return get_exe_for_pid(pid);
                }
            }
        }
    }
    let pid = get_active_app_pid_fallbacks()?;
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
                let mut x11_conn = None;

                while running.load(Ordering::SeqCst) {
                    if x11_conn.is_none() {
                        if let Ok(display) = std::env::var("DISPLAY") {
                            if !display.is_empty() {
                                if let Ok(conn_pair) = x11rb::connect(None) {
                                    x11_conn = Some(conn_pair);
                                }
                            }
                        }
                    }

                    let mut pid = None;

                    if let Some((ref conn, screen_num)) = x11_conn {
                        match get_active_app_pid_x11_persistent(conn, screen_num) {
                            Ok(Some(p)) => {
                                pid = Some(p);
                            }
                            Ok(None) => {}
                            Err(_) => {
                                // Connection closed or failed, clear it to reconnect next iteration
                                x11_conn = None;
                            }
                        }
                    }

                    if pid.is_none() {
                        pid = get_active_app_pid_fallbacks();
                    }

                    match pid {
                        Some(p) => {
                            if let Some(exe) = get_exe_for_pid(p) {
                                if exe != last_exe {
                                    last_exe = exe.clone();
                                    on_change(exe);
                                }
                            }
                        }
                        None => {
                            if !tool_warned {
                                log::info!(
                                    "[AppDetector] Active window connection is not an XWayland client or GNOME D-Bus query returned empty. \
                                     Per-app profile switching falls back to default for native Wayland windows."
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
