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

fn get_active_app_pid_x11() -> Option<u32> {
    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Get _NET_ACTIVE_WINDOW
    let active_window_atom = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW").ok()?.reply().ok()?.atom;
    let pid_atom = conn.intern_atom(false, b"_NET_WM_PID").ok()?.reply().ok()?.atom;

    let active_window_resp = conn.get_property(false, root, active_window_atom, AtomEnum::WINDOW, 0, 1).ok()?.reply().ok()?;
    if active_window_resp.value_len == 0 {
        return None;
    }
    let window_id = active_window_resp.value32()?.next()?;

    // Get _NET_WM_PID of the active window
    let pid_resp = conn.get_property(false, window_id, pid_atom, AtomEnum::CARDINAL, 0, 1).ok()?.reply().ok()?;
    if pid_resp.value_len == 0 {
        return None;
    }
    let pid = pid_resp.value32()?.next();
    pid
}

fn get_active_app_pid() -> Option<u32> {
    let is_wayland = std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_lowercase()
        == "wayland";

    let is_kde = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_uppercase()
        .contains("KDE");

    if is_wayland && is_kde {
        if let Some(pid) = get_pid_from_cmd("kdotool") {
            return Some(pid);
        }
    } else if !is_wayland {
        if let Some(pid) = get_active_app_pid_x11() {
            return Some(pid);
        }
        // Fallback to xdotool if x11rb method fails
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
                                    "[AppDetector] Could not detect active window PID. Ensure X11/KDE/xdotool is available."
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
