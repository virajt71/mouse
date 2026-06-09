use super::gnome::get_active_app_pid_gnome_shell;
use super::kde::get_pid_from_kdotool;
use super::xdotool::get_pid_from_xdotool;
use super::sway::get_active_app_pid_sway;
use super::hyprland::get_active_app_pid_hyprland;
use super::i3::get_active_app_pid_i3;

pub fn get_active_app_pid_fallbacks(x11_queried: bool) -> Option<u32> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_uppercase();
    let session_type = std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_lowercase();

    let is_gnome     = desktop.contains("GNOME");
    let is_kde       = desktop.contains("KDE");
    let is_sway      = desktop.contains("SWAY");
    let is_hyprland  = desktop.contains("HYPRLAND");
    let is_i3        = desktop.contains("I3");
    let has_display  = std::env::var("DISPLAY").map(|v| !v.is_empty()).unwrap_or(false);
    let is_wayland   = session_type == "wayland"
        || std::env::var("WAYLAND_DISPLAY").map(|v| !v.is_empty()).unwrap_or(false);

    // 1. GNOME Shell D-Bus Eval (GNOME ≤44 Wayland native apps)
    if is_gnome && is_wayland {
        if let Some(pid) = get_active_app_pid_gnome_shell() {
            return Some(pid);
        }
    }

    // 2. kdotool — KDE Wayland native apps
    if is_kde && is_wayland {
        if let Some(pid) = get_pid_from_kdotool() {
            return Some(pid);
        }
    }

    // 3. Sway
    if is_sway {
        if let Some(pid) = get_active_app_pid_sway() {
            return Some(pid);
        }
    }

    // 4. Hyprland
    if is_hyprland {
        if let Some(pid) = get_active_app_pid_hyprland() {
            return Some(pid);
        }
    }

    // 5. i3 (X11, but JSON path is more reliable than xdotool on i3)
    if is_i3 {
        if let Some(pid) = get_active_app_pid_i3() {
            return Some(pid);
        }
    }

    // 6. xdotool — legacy X11 / XWayland subprocess fallback
    if has_display && !x11_queried {
        if let Some(pid) = get_pid_from_xdotool() {
            return Some(pid);
        }
    }

    None
}
