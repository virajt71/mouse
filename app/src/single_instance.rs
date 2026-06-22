use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

pub fn get_socket_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config")
    });
    path.push("Mouser");
    let _ = std::fs::create_dir_all(&path);
    path.push("mouser.sock");
    path
}

pub fn setup_single_instance(daemon_mode: bool) -> Option<UnixListener> {
    let socket_path = get_socket_path();
    match UnixListener::bind(&socket_path) {
        Ok(listener) => {
            // Set socket permissions to 0600 (read/write only by the owner)
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600));
            Some(listener)
        }
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
                            eprintln!(
                                "Failed to bind to socket after removing stale file: {}",
                                err
                            );
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
