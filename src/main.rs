// engine and gui are external library crates in the workspace
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
use eframe::egui;

use std::path::PathBuf;
use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{Write, Read};

static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn handle_sig(_sig: std::os::raw::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

fn get_socket_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config")
    });
    path.push("Mouser");
    let _ = std::fs::create_dir_all(&path);
    path.push("mouser.sock");
    path
}

fn setup_single_instance(daemon_mode: bool) -> Option<UnixListener> {
    let socket_path = get_socket_path();
    match UnixListener::bind(&socket_path) {
        Ok(listener) => Some(listener),
        Err(ref e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            match UnixStream::connect(&socket_path) {
                Ok(mut stream) => {
                    if !daemon_mode {
                        let _ = stream.write_all(b"SHOW");
                        println!("Another instance of Mouser is already running. Showing existing window.");
                    } else {
                        println!("Another instance of Mouser is already running.");
                    }
                    None
                }
                Err(_) => {
                    // Stale socket
                    let _ = std::fs::remove_file(&socket_path);
                    match UnixListener::bind(&socket_path) {
                        Ok(listener) => Some(listener),
                        Err(err) => {
                            eprintln!("Failed to bind to socket after removing stale file: {}", err);
                            None
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to bind to socket: {}", e);
            None
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = std::env::args().collect();
    let mut debug = false;
    let mut daemon_mode = false;
    for arg in &args[1..] {
        if arg == "--debug" || arg == "-d" {
            debug = true;
        } else if arg == "--daemon" {
            daemon_mode = true;
        } else if arg == "--help" || arg == "-h" {
            println!(
                "Mouser Rust Linux Daemon\n\
                 Usage: mouser-rs [options]\n\
                 Options:\n\
                   -d, --debug    Enable debug level logging\n\
                   --daemon       Run headless in background mode (no GUI)\n\
                   -h, --help     Show this help message"
            );
            return Ok(());
        }
    }

    if std::env::var("RUST_LOG").is_err() {
        if debug {
            std::env::set_var("RUST_LOG", "debug");
        } else {
            std::env::set_var("RUST_LOG", "info,zbus=warn,tracing=warn,sctk_adwaita=off");
        }
    }

    let log_dir = engine::config::get_log_dir();
    let _logger = flexi_logger::Logger::try_with_env_or_str("info,zbus=warn,tracing=warn,sctk_adwaita=off")
        .expect("Failed to parse log configuration")
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory(log_dir)
                .basename("mouser")
                .suffix("log"),
        )
        .rotate(
            flexi_logger::Criterion::Size(5 * 1024 * 1024), // 5 MB
            flexi_logger::Naming::Numbers,
            flexi_logger::Cleanup::KeepLogFiles(5), // Keep 5 files
        )
        .duplicate_to_stdout(flexi_logger::Duplicate::All) // Show in stdout/stderr as well
        .start()
        .expect("Failed to initialize logger");

    log::info!("Starting Mouser Rust Daemon...");

    unsafe {
        let sa = SigAction::new(SigHandler::Handler(handle_sig), SaFlags::empty(), SigSet::empty());
        let _ = sigaction(Signal::SIGINT, &sa);
        let _ = sigaction(Signal::SIGTERM, &sa);
        let _ = sigaction(Signal::SIGHUP, &sa);
    }

    let listener = match setup_single_instance(daemon_mode) {
        Some(l) => l,
        None => return Ok(()),
    };

    let engine = engine::Engine::new();
    if let Err(e) = engine.start() {
        log::error!("Failed to start Mouser engine: {}", e);
        if daemon_mode {
            let _ = std::fs::remove_file(get_socket_path());
            return Ok(());
        }
    }

    if daemon_mode {
        // Spawn a thread to consume UDS socket connections
        let socket_path_clone = get_socket_path();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    let mut buf = [0; 64];
                    let _ = stream.read(&mut buf);
                }
            }
        });

        while RUNNING.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }
        engine.stop();
        let _ = std::fs::remove_file(socket_path_clone);
        Ok(())
    } else {
        run_gui(engine, listener)
    }
}

fn run_gui(engine: engine::Engine, listener: UnixListener) -> Result<(), eframe::Error> {
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
            #[cfg(target_os = "linux")]
            let tray_icon = {
                // Spawn a dedicated thread for GTK tray icon initialization and event loop
                let engine_clone = engine.clone();
                std::thread::spawn({
                    let ctx = cc.egui_ctx.clone();
                    move || {
                        gtk::glib::log_set_default_handler(|_, _, _| {});
                        gtk::init().expect("Failed to initialize GTK for tray icon");

                        use tray_icon::menu::{Menu, MenuItemBuilder, PredefinedMenuItem};
                        let tray_menu = Menu::new();
                        let open_item = MenuItemBuilder::new()
                            .text("Open Mouser-RS")
                            .enabled(true)
                            .build();
                        let quit_item = MenuItemBuilder::new().text("Quit").enabled(true).build();
                        tray_menu.append(&open_item).unwrap();
                        tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
                        tray_menu.append(&quit_item).unwrap();

                        let open_id = open_item.id().clone();
                        let quit_id = quit_item.id().clone();

                        let _tray = tray_icon::TrayIconBuilder::new()
                            .with_menu(Box::new(tray_menu))
                            .with_tooltip("Mouser-rs")
                            .with_icon(gui::theme::create_mouse_tray_icon())
                            .build()
                            .expect("Failed to build tray icon");

                        // Spawn menu event listener thread
                        let engine_q = engine_clone.clone();
                        std::thread::spawn(move || {
                            let menu_channel = tray_icon::menu::MenuEvent::receiver();
                            loop {
                                if let Ok(event) = menu_channel.recv() {
                                    if event.id == open_id {
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                                        if let Some(monitor) =
                                            ctx.input(|i| i.viewport().monitor_size)
                                        {
                                            gui::theme::center_window(&ctx, monitor);
                                        }
                                    } else if event.id == quit_id {
                                        engine_q.stop();
                                        std::process::exit(0);
                                    }
                                }
                            }
                        });

                        // Run GTK main loop to keep the tray icon alive and responsive
                        gtk::main();
                    }
                });
                None
            };

            #[cfg(not(target_os = "linux"))]
            let tray_icon = {
                use tray_icon::menu::{Menu, MenuItemBuilder, PredefinedMenuItem};
                let tray_menu = Menu::new();
                let open_item = MenuItemBuilder::new()
                    .text("Open Application (Mouser-rs)")
                    .enabled(true)
                    .build();
                let quit_item = MenuItemBuilder::new().text("Quit").enabled(true).build();
                tray_menu.append(&open_item).unwrap();
                tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
                tray_menu.append(&quit_item).unwrap();

                let open_id = open_item.id().clone();
                let quit_id = quit_item.id().clone();

                let icon = tray_icon::TrayIconBuilder::new()
                    .with_menu(Box::new(tray_menu))
                    .with_tooltip("Mouser-rs")
                    .with_icon(gui::theme::create_mouse_tray_icon())
                    .build()
                    .unwrap();

                let engine_q = engine.clone();
                std::thread::spawn({
                    let ctx = cc.egui_ctx.clone();
                    move || {
                        let menu_channel = tray_icon::menu::MenuEvent::receiver();
                        loop {
                            if let Ok(event) = menu_channel.recv() {
                                if event.id == open_id {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                                    if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size)
                                    {
                                        gui::theme::center_window(&ctx, monitor);
                                    }
                                } else if event.id == quit_id {
                                    engine_q.stop();
                                    std::process::exit(0);
                                }
                            }
                        }
                    }
                });
                Some(icon)
            };

            // Spawn single-instance UDS message listener
            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    if let Ok(mut stream) = stream {
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
                }
            });

            Ok(Box::new(gui::MouserApp::new(
                cc.egui_ctx.clone(),
                tray_icon,
                engine.clone(),
            )))
        }),
    ).map(|_| {
        engine_for_close.stop();
        let _ = std::fs::remove_file(get_socket_path());
    })
}

