use crate::single_instance::get_socket_path;
use crate::tray::setup_tray;
use eframe::egui;
use engine::Engine;
use std::io::Read;
use std::os::unix::net::UnixListener;

pub fn run_gui(engine: Engine, listener: UnixListener) -> Result<(), eframe::Error> {
    // Under WSL2, Mesa's hardware acceleration can fail with Zink driver/EGL errors.
    // Force software rendering fallback if running inside WSL to ensure out-of-the-box compatibility.
    #[cfg(target_os = "linux")]
    if std::env::var("WSL_DISTRO_NAME").is_ok() && std::env::var("LIBGL_ALWAYS_SOFTWARE").is_err() {
        std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
    }

    // Force X11/XWayland backend on Linux to ensure set_visible(false) is supported (Wayland does not support dynamic window hiding in winit).
    #[cfg(target_os = "linux")]
    std::env::remove_var("WAYLAND_DISPLAY");

    // Silence GLib, GIO, and GTK console logging warnings globally on Linux.
    #[cfg(target_os = "linux")]
    gtk::glib::log_set_default_handler(|_, _, _| {});

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_resizable(false)
            .with_inner_size([gui::theme::WINDOW_W, gui::theme::WINDOW_H]),
        ..Default::default()
    };

    let engine_for_close = engine.clone();
    eframe::run_native(
        "Mouser-rs",
        native_options,
        Box::new(move |cc| {
            gui::theme::setup_fonts(&cc.egui_ctx);

            // Center on the primary monitor at startup
            if let Some(monitor) = cc.egui_ctx.input(|i| i.viewport().monitor_size) {
                gui::theme::center_window(&cc.egui_ctx, monitor);
            }

            // System Tray Initialization
            let tray_icon = setup_tray(cc.egui_ctx.clone(), engine.clone());

            // Spawn single-instance UDS message listener
            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                for mut stream in listener.incoming().flatten() {
                    let mut buf = [0; 64];
                    if let Ok(n) = stream.read(&mut buf) {
                        let msg = String::from_utf8_lossy(&buf[..n]);
                        if msg.trim() == "SHOW" {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                            if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
                                gui::theme::center_window(&ctx, monitor);
                            }
                        }
                    }
                }
            });

            Ok(Box::new(gui::MouserApp::new(
                cc.egui_ctx.clone(),
                tray_icon,
                engine.clone(),
            )))
        }),
    )
    .map(|_| {
        engine_for_close.stop();
        let _ = std::fs::remove_file(get_socket_path());
    })
}
