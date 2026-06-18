pub mod fallbacks;
pub mod gnome;
pub mod hyprland;
pub mod i3;
pub mod kde;
pub mod sway;
pub mod thread;
pub mod x11;
pub mod xdotool;

pub use self::thread::AppDetector;

use self::fallbacks::get_active_app_pid_fallbacks;
use self::x11::get_active_app_pid_x11_persistent;

pub fn get_exe_for_pid(pid: u32) -> Option<String> {
    let path = format!("/proc/{}/exe", pid);
    if let Ok(target) = std::fs::read_link(path) {
        if let Some(filename) = target.file_name() {
            return Some(filename.to_string_lossy().to_string());
        }
    }
    None
}

static LAST_PID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
thread_local! {
    pub static X11_CONN: std::cell::RefCell<Option<(x11rb::rust_connection::RustConnection, usize)>> = const { std::cell::RefCell::new(None) };
    static LAST_EXE: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

fn get_exe_for_pid_cached(pid: u32) -> Option<String> {
    let last_pid = LAST_PID.load(std::sync::atomic::Ordering::Relaxed);
    if pid == last_pid {
        let cached = LAST_EXE.with(|e| e.borrow().clone());
        if !cached.is_empty() {
            return Some(cached);
        }
    }
    if let Some(exe) = get_exe_for_pid(pid) {
        LAST_PID.store(pid, std::sync::atomic::Ordering::Relaxed);
        LAST_EXE.with(|e| *e.borrow_mut() = exe.clone());
        Some(exe)
    } else {
        None
    }
}

pub fn get_foreground_exe() -> Option<String> {
    if let Ok(display) = std::env::var("DISPLAY") {
        if !display.is_empty() {
            let pid = X11_CONN.with(|cell| {
                let mut opt = cell.borrow_mut();
                if opt.is_none() {
                    if let Ok(conn_pair) = x11rb::connect(None) {
                        *opt = Some(conn_pair);
                    }
                }
                if let Some((ref conn, screen_num)) = *opt {
                    match get_active_app_pid_x11_persistent(conn, screen_num) {
                        Ok(Some(pid)) => Some(pid),
                        Ok(None) => None,
                        Err(_) => {
                            // Connection failed, clear from cache
                            *opt = None;
                            None
                        }
                    }
                } else {
                    None
                }
            });
            if let Some(p) = pid {
                return get_exe_for_pid_cached(p);
            }
        }
    }
    let pid = get_active_app_pid_fallbacks(X11_CONN.with(|cell| cell.borrow().is_some()))?;
    get_exe_for_pid_cached(pid)
}
