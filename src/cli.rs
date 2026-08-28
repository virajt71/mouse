use engine::client::EngineClient;
use engine::config::get_grpc_socket_path;
use std::error::Error;

// ponytail: Basic CLI argument parsing via std::env::args matcher.
// Known ceiling: No complex flag combinations or autocompletion.
// Upgrade path: Migrate to `clap` crate if flag complexity increases.

pub fn print_help() {
    println!(
        "Mouser-RS Command Line Interface\n\n\
         USAGE:\n    \
           mouser-rs <SUBCOMMAND> [OPTIONS]\n\n\
         SUBCOMMANDS:\n    \
           status, info          Display daemon state, device info, battery %, active profile, and DPI\n    \
           profile list          List available application profiles in active group\n    \
           profile get           Print currently active application profile name\n    \
           profile set <name>    Switch active application profile\n    \
           dpi get               Get current configured DPI value\n    \
           dpi set <value>       Set mouse DPI value (e.g. 800, 1000, 1600, 3200)\n    \
           reload                Trigger background daemon configuration hot-reload\n    \
           help, --help, -h      Show this help documentation\n"
    );
}

pub fn run_cli(args: &[String]) -> Result<(), Box<dyn Error>> {
    if args.is_empty() {
        print_help();
        return Ok(());
    }

    let command = args[0].as_str();

    match command {
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "status" | "info" => handle_status(),
        "profile" => {
            if args.len() < 2 {
                println!("Error: 'profile' requires a subcommand ('list', 'get', or 'set <name>')");
                return Ok(());
            }
            match args[1].as_str() {
                "list" => handle_profile_list(),
                "get" => handle_profile_get(),
                "set" => {
                    if args.len() < 3 {
                        println!("Error: 'profile set' requires a profile name argument");
                        return Ok(());
                    }
                    handle_profile_set(&args[2])
                }
                sub => {
                    println!(
                        "Unknown profile subcommand '{}'. Use 'list', 'get', or 'set'.",
                        sub
                    );
                    Ok(())
                }
            }
        }
        "dpi" => {
            if args.len() < 2 {
                println!("Error: 'dpi' requires a subcommand ('get' or 'set <value>')");
                return Ok(());
            }
            match args[1].as_str() {
                "get" => handle_dpi_get(),
                "set" => {
                    if args.len() < 3 {
                        println!("Error: 'dpi set' requires a numeric value argument (e.g. 1000)");
                        return Ok(());
                    }
                    if let Ok(dpi_val) = args[2].parse::<i32>() {
                        handle_dpi_set(dpi_val)
                    } else {
                        println!(
                            "Error: Invalid DPI value '{}'. Must be an integer.",
                            args[2]
                        );
                        Ok(())
                    }
                }
                sub => {
                    println!("Unknown dpi subcommand '{}'. Use 'get' or 'set'.", sub);
                    Ok(())
                }
            }
        }
        "reload" => handle_reload(),
        unknown => {
            println!(
                "Unknown command '{}'. Run 'mouser-rs help' for available commands.",
                unknown
            );
            Ok(())
        }
    }
}

fn connect_client() -> Result<EngineClient, Box<dyn Error>> {
    let socket_path = get_grpc_socket_path();
    match EngineClient::connect(&socket_path) {
        Ok(client) => Ok(client),
        Err(_) => {
            Err(format!(
                "Failed to connect to Mouser daemon socket at {:?}.\nEnsure Mouser daemon is running (`mouser-rs --daemon` or start GUI).",
                socket_path
            ).into())
        }
    }
}

fn handle_status() -> Result<(), Box<dyn Error>> {
    let client = connect_client()?;
    let config = client.get_config();
    let connected = client.device_connected();
    let dev_name = client.selected_device_name();

    println!("=== Mouser-RS System Status ===");
    println!("Daemon Status  : Connected (gRPC)");
    println!(
        "Device Status  : {}",
        if connected {
            "Connected"
        } else {
            "Disconnected / Standby"
        }
    );
    println!("Device Name    : {}", dev_name);
    println!("Active Group   : {}", config.active_group);
    println!("Active Profile : {}", config.active_app_profile);
    println!("DPI Setting    : {}", config.settings.dpi);
    println!(
        "SmartShift     : {} (Mode: {}, Threshold: {})",
        if config.settings.smart_shift_enabled {
            "Enabled"
        } else {
            "Disabled"
        },
        config.settings.smart_shift_mode,
        config.settings.smart_shift_threshold
    );
    println!(
        "Flow Network   : {}",
        if config.settings.flow_enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    Ok(())
}

fn handle_profile_list() -> Result<(), Box<dyn Error>> {
    let client = connect_client()?;
    let config = client.get_config();

    if let Some(group) = config.profile_groups.get(&config.active_group) {
        println!("Profiles in group '{}':", config.active_group);
        for (pname, pdata) in &group.profiles {
            let active_marker = if pname == &config.active_app_profile {
                " (active)"
            } else {
                ""
            };
            let apps_str = if pdata.apps.is_empty() {
                "All Applications".to_string()
            } else {
                pdata.apps.join(", ")
            };
            println!("  - {} [{}]", pname, apps_str);
            println!("    Label: {}{}", pdata.label, active_marker);
        }
    } else {
        println!(
            "No profiles found in active group '{}'.",
            config.active_group
        );
    }
    Ok(())
}

fn handle_profile_get() -> Result<(), Box<dyn Error>> {
    let client = connect_client()?;
    let config = client.get_config();
    println!("{}", config.active_app_profile);
    Ok(())
}

fn handle_profile_set(name: &str) -> Result<(), Box<dyn Error>> {
    let client = connect_client()?;
    let config = client.get_config();

    if let Some(group) = config.profile_groups.get(&config.active_group) {
        if !group.profiles.contains_key(name) {
            println!(
                "Warning: Profile '{}' does not exist in group '{}'. Setting anyway.",
                name, config.active_group
            );
        }
    }

    client.select_profile(name);
    println!("Active profile updated to '{}'.", name);
    Ok(())
}

fn handle_dpi_get() -> Result<(), Box<dyn Error>> {
    let client = connect_client()?;
    let config = client.get_config();
    println!("{}", config.settings.dpi);
    Ok(())
}

fn handle_dpi_set(dpi: i32) -> Result<(), Box<dyn Error>> {
    let client = connect_client()?;
    let mut config = client.get_config();
    config.settings.dpi = dpi;

    client.update_global_settings(
        dpi as u32,
        config.settings.smart_shift_mode,
        config.settings.smart_shift_enabled,
        config.settings.smart_shift_threshold as u8,
        config.settings.invert_hscroll,
        config.settings.invert_vscroll,
        config.settings.gesture_threshold,
        config.settings.gesture_deadzone,
        config.settings.accent_color,
        config.settings.hscroll_threshold,
    );

    println!("DPI updated to {}.", dpi);
    Ok(())
}

fn handle_reload() -> Result<(), Box<dyn Error>> {
    let client = connect_client()?;
    client.reload_config();
    println!("Mouser daemon configuration reloaded.");
    Ok(())
}
