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
                        pid = get_active_app_pid_fallbacks(x11_conn.is_some());
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
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
