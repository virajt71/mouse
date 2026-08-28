// engine and gui are external library crates in the workspace
use std::thread;
use std::time::Duration;

mod cli;
mod gui;
mod signal;
mod single_instance;
mod tray;

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = std::env::args().collect();
    let mut debug = false;
    let mut daemon_mode = false;

    // Check for daemon or debug flags
    for arg in &args[1..] {
        if arg == "--debug" || arg == "-d" {
            debug = true;
        } else if arg == "--daemon" {
            daemon_mode = true;
        }
    }

    // If subcommands are provided and not in daemon mode, execute CLI
    if !daemon_mode && args.len() > 1 {
        let first_arg = args[1].as_str();
        if matches!(
            first_arg,
            "status" | "info" | "profile" | "dpi" | "reload" | "cli" | "help" | "--help" | "-h"
        ) {
            let cli_args = if first_arg == "cli" {
                &args[2..]
            } else {
                &args[1..]
            };
            if let Err(err) = cli::run_cli(cli_args) {
                eprintln!("{}", err);
                std::process::exit(1);
            }
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
            .duplicate_to_stdout(if debug {
                flexi_logger::Duplicate::All
            } else {
                flexi_logger::Duplicate::None
            })
            .start()
            .expect("Failed to initialize logger");

    log::info!("Starting Mouser Rust Daemon...");

    signal::setup_signal_handlers();

    let grpc_socket_path = engine::config::get_grpc_socket_path();

    if daemon_mode {
        log::info!("Running in headless daemon mode...");
        let engine = engine::Engine::new();
        if let Err(e) = engine.start() {
            log::error!("Failed to start Mouser engine: {}", e);
            return Ok(());
        }

        let (config_bc, device_state_bc) =
            match engine::grpc::start_grpc_server(engine.clone(), &grpc_socket_path) {
                Ok(res) => res,
                Err(e) => {
                    log::error!("Failed to start gRPC server: {}", e);
                    engine.stop();
                    return Ok(());
                }
            };

        // Broadcast config changes over gRPC
        let config_bc_clone = config_bc.clone();
        engine.set_config_change_listener(move |cfg, gen| {
            engine::grpc::broadcast_config(cfg, gen, &config_bc_clone);
        });

        // Broadcast device state changes over gRPC
        let dev_bc_clone = device_state_bc.clone();
        let (_worker_tx, worker_rx) =
            engine::worker::spawn_background_worker(move || {}, engine.active_profile_shared());
        std::thread::spawn(move || {
            while let Ok(update) = worker_rx.recv() {
                engine::grpc::broadcast_device_state(&update, &dev_bc_clone);
            }
        });

        log::info!("Mouser daemon successfully started and listening for gRPC clients.");

        while signal::is_running() {
            thread::sleep(Duration::from_millis(100));
        }

        log::info!("Shutting down Mouser daemon...");
        engine.stop();
        let _ = std::fs::remove_file(&grpc_socket_path);
        Ok(())
    } else {
        // Single instance check for GUI app
        let listener = match single_instance::setup_single_instance(false) {
            Some(l) => l,
            None => return Ok(()),
        };

        // Try connecting to existing daemon via gRPC
        let mut client = engine::client::EngineClient::connect(&grpc_socket_path).ok();

        // If daemon is not running, spawn it in background automatically
        if client.is_none() {
            log::info!("Daemon not running. Auto-starting Mouser daemon...");
            if let Ok(exe) = std::env::current_exe() {
                let _ = std::process::Command::new(exe).arg("--daemon").spawn();
            }

            // Retry connecting to daemon with backoff
            for attempt in 1..=30 {
                thread::sleep(Duration::from_millis(100));
                if let Ok(c) = engine::client::EngineClient::connect(&grpc_socket_path) {
                    log::info!("Connected to Mouser daemon after attempt {}", attempt);
                    client = Some(c);
                    break;
                }
            }
        }

        let client = match client {
            Some(c) => c,
            None => {
                log::error!(
                    "Failed to connect to Mouser daemon socket: {:?}",
                    grpc_socket_path
                );
                return Ok(());
            }
        };

        gui::run_gui(client, listener)
    }
}
