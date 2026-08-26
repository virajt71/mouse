use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::fallbacks::get_active_app_pid_fallbacks;
use super::get_exe_for_pid;
use super::x11::get_active_app_pid_x11_persistent;

pub struct AppDetector {
    on_change: Arc<dyn Fn(String) + Send + Sync + 'static>,
    interval: Duration,
    running: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
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
        if self.running.load(Ordering::Acquire) {
            return;
        }

        self.running.store(true, Ordering::Release);
        let running = self.running.clone();
        let interval = self.interval;
        let on_change = self.on_change.clone();

        let handle = thread::Builder::new()
            .name("AppDetector".to_string())
            .spawn(move || {
                let mut last_exe = String::new();
                let mut tool_warned = false;
                let mut x11_conn = None;

                let session_type = std::env::var("XDG_SESSION_TYPE")
                    .unwrap_or_default()
                    .to_lowercase();
                let is_wayland = session_type == "wayland"
                    || std::env::var("WAYLAND_DISPLAY")
                        .map(|v| !v.is_empty())
                        .unwrap_or(false);

                let mut wlr_tracker = None;
                let mut gnome_tracker = None;
                let mut retry_counter: u32 = 0;

                while running.load(Ordering::Acquire) {
                    if is_wayland {
                        if wlr_tracker.is_none() && (retry_counter == 0 || retry_counter % 10 == 0) {
                            wlr_tracker = super::wlr_foreign_toplevel::WlrForeignToplevelTracker::connect();
                        }
                        if wlr_tracker.is_none() && gnome_tracker.is_none() && (retry_counter == 0 || retry_counter % 10 == 0) {
                            gnome_tracker = super::gnome::GnomeDbusTracker::connect();
                        }
                        retry_counter = retry_counter.wrapping_add(1);
                    }

                    let mut app_name = None;

                    // 1. Try native Wayland tracker (wlroots)
                    if let Some(ref tracker) = wlr_tracker {
                        if let Some(app_id) = tracker.get_foreground_app_id() {
                            app_name = Some(super::normalize_app_id(&app_id));
                        }
                    }

                    // 2. Try native GNOME D-Bus tracker
                    if app_name.is_none() {
                        if let Some(ref tracker) = gnome_tracker {
                            if let Some(res) = tracker.get_foreground_app() {
                                match res {
                                    Ok(wm_class) => {
                                        app_name = Some(super::normalize_app_id(&wm_class));
                                    }
                                    Err(pid) => {
                                        if let Some(exe) = get_exe_for_pid(pid) {
                                            app_name = Some(exe);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 3. Try X11 persistent connection (for X11 and XWayland windows)
                    if app_name.is_none() {
                        if x11_conn.is_none() {
                            if let Ok(display) = std::env::var("DISPLAY") {
                                if !display.is_empty() {
                                    if let Ok(conn_pair) = x11rb::connect(None) {
                                        x11_conn = Some(conn_pair);
                                    }
                                }
                            }
                        }
                        if let Some((ref conn, screen_num)) = x11_conn {
                            match get_active_app_pid_x11_persistent(conn, screen_num) {
                                Ok(Some(pid)) => {
                                    if let Some(exe) = get_exe_for_pid(pid) {
                                        app_name = Some(exe);
                                    }
                                }
                                Ok(None) => {}
                                Err(_) => {
                                    // Connection failed, clear it to reconnect next iteration
                                    x11_conn = None;
                                }
                            }
                        }
                    }

                    // 4. Try CLI tools as fallbacks
                    if app_name.is_none() {
                        if let Some(pid) = get_active_app_pid_fallbacks(x11_conn.is_some()) {
                            if let Some(exe) = get_exe_for_pid(pid) {
                                app_name = Some(exe);
                            }
                        }
                    }

                    match app_name {
                        Some(exe) => {
                            if exe != last_exe {
                                last_exe = exe.clone();
                                on_change(exe);
                            }
                        }
                        None => {
                            if !tool_warned {
                                log::debug!(
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
        if !self.running.load(Ordering::Acquire) {
            return;
        }

        self.running.store(false, Ordering::Release);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
