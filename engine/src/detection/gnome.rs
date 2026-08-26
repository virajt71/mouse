use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use zbus::blocking::Connection;

pub static GNOME_DBUS_SUPPORTED: AtomicBool = AtomicBool::new(true);

const METHOD_TIMEOUT: Duration = Duration::from_secs(2);

#[zbus::proxy(
    interface = "org.mouser.Frontmost",
    default_service = "org.mouser.Frontmost",
    default_path = "/org/mouser/Frontmost",
    gen_async = false
)]
trait MouserFrontmost {
    #[zbus(name = "GetFocusedWmClass")]
    fn get_focused_wm_class(&self) -> zbus::Result<String>;

    #[zbus(name = "GetFocusedWindowInfo")]
    fn get_focused_window_info(&self) -> zbus::Result<(String, u32)>;
}

#[zbus::proxy(
    interface = "org.openlogi.Frontmost",
    default_service = "org.openlogi.Frontmost",
    default_path = "/org/openlogi/Frontmost",
    gen_async = false
)]
trait OpenLogiFrontmost {
    #[zbus(name = "GetFocusedWmClass")]
    fn get_focused_wm_class(&self) -> zbus::Result<String>;
}

#[zbus::proxy(
    interface = "org.gnome.Shell",
    default_service = "org.gnome.Shell",
    default_path = "/org/gnome/Shell",
    gen_async = false
)]
trait GnomeShell {
    #[zbus(name = "Eval")]
    fn eval(&self, code: &str) -> zbus::Result<(bool, String)>;
}

pub struct GnomeDbusTracker {
    conn: Connection,
}

impl GnomeDbusTracker {
    pub fn connect() -> Option<Self> {
        if !GNOME_DBUS_SUPPORTED.load(Ordering::Relaxed) {
            return None;
        }
        let conn = zbus::blocking::connection::Builder::session()
            .map_err(|e| log::debug!("[GnomeTracker] Session builder failed: {}", e))
            .ok()?
            .method_timeout(METHOD_TIMEOUT)
            .build()
            .map_err(|e| log::debug!("[GnomeTracker] Session connection build failed: {}", e))
            .ok()?;
        Some(Self { conn })
    }

    /// Try to query the foreground app name.
    /// Returns either the WM_CLASS (from extensions) or the PID (from GetFocusedWindowInfo / Eval).
    pub fn get_foreground_app(&self) -> Option<Result<String, u32>> {
        // 1. Try native Mouser companion extension (returns String and PID)
        if let Ok(proxy) = MouserFrontmostProxy::new(&self.conn) {
            // Try new method first to get PID for precise exe matching
            if let Ok((wm_class, pid)) = proxy.get_focused_window_info() {
                if pid > 0 {
                    return Some(Err(pid));
                } else if !wm_class.is_empty() {
                    return Some(Ok(wm_class));
                }
            }
            // Fallback to legacy single method if client has older extension version loaded
            if let Ok(wm_class) = proxy.get_focused_wm_class() {
                if !wm_class.is_empty() {
                    return Some(Ok(wm_class));
                }
            }
        }

        // 2. Try OpenLogi companion extension as a fallback (returns String)
        if let Ok(proxy) = OpenLogiFrontmostProxy::new(&self.conn) {
            if let Ok(wm_class) = proxy.get_focused_wm_class() {
                if !wm_class.is_empty() {
                    return Some(Ok(wm_class));
                }
            }
        }

        // 3. Try org.gnome.Shell.Eval (returns PID as u32)
        if let Ok(proxy) = GnomeShellProxy::new(&self.conn) {
            match proxy.eval("global.display.focus_window?.get_pid() ?? -1") {
                Ok((true, pid_str)) => {
                    let pid_val: i64 = pid_str.trim().parse().ok()?;
                    if pid_val > 0 {
                        return Some(Err(pid_val as u32));
                    }
                }
                Ok((false, _)) => {
                    // Method succeeded but JS execution failed
                }
                Err(e) => {
                    // Eval API might be disabled
                    log::debug!("[GnomeTracker] GNOME Shell Eval failed: {}", e);
                }
            }
        }

        None
    }
}
