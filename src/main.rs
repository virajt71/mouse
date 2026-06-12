// engine and gui are external library crates in the workspace
use std::io::Read;
use std::thread;
use std::time::Duration;

mod gui;
mod signal;
mod single_instance;
mod tray;

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
    let _logger =
        flexi_logger::Logger::try_with_env_or_str("info,zbus=warn,tracing=warn,sctk_adwaita=off")
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

    signal::setup_signal_handlers();

    let listener = match single_instance::setup_single_instance(daemon_mode) {
        Some(l) => l,
        None => return Ok(()),
    };

    let engine = engine::Engine::new();
    if let Err(e) = engine.start() {
        log::error!("Failed to start Mouser engine: {}", e);
        if daemon_mode {
            let _ = std::fs::remove_file(single_instance::get_socket_path());
            return Ok(());
        }
    }

    if daemon_mode {
        // Spawn a thread to consume UDS socket connections
        let socket_path_clone = single_instance::get_socket_path();
        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut buf = [0; 64];
                let _ = stream.read(&mut buf);
            }
        });

        while signal::is_running() {
            thread::sleep(Duration::from_millis(100));
        }
        engine.stop();
        let _ = std::fs::remove_file(socket_path_clone);
        Ok(())
    } else {
        gui::run_gui(engine, listener)
    }
}
