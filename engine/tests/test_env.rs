
use std::process::Command;

#[test]
fn test_env_vars() {
    println!("MOUSER_CONFIG_PATH={}", std::env::var("MOUSER_CONFIG_PATH").unwrap_or("NOT SET".to_string()));
    println!("All env vars:");
    for (k, v) in std::env::vars() {
        if k.starts_with("MOUSER") || k.starts_with("TEST") || k.starts_with("CARGO") {
            println!("{}={}", k, v);
        }
    }
    println!("Current dir: {}", std::env::current_dir().unwrap().display());
}
