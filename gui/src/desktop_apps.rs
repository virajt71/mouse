use std::fs;
use std::path::PathBuf;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopApp {
    pub name: String,
    pub exec: String,
    pub icon: String,
    pub path: String,
}

pub fn scan_desktop_applications() -> Vec<DesktopApp> {
    let mut apps = Vec::new();
    let mut dirs_to_scan = vec![PathBuf::from("/usr/share/applications")];
    if let Some(home) = dirs::home_dir() {
        dirs_to_scan.push(home.join(".local/share/applications"));
    }

    let mut seen_names = HashSet::new();

    for dir in dirs_to_scan {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "desktop") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Some(app) = parse_desktop_file(&content, path.to_string_lossy().into_owned()) {
                            let name_lower = app.name.to_lowercase();
                            if !seen_names.contains(&name_lower) {
                                seen_names.insert(name_lower);
                                apps.push(app);
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort alphabetically by name
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

fn parse_desktop_file(content: &str, path: String) -> Option<DesktopApp> {
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut no_display = false;
    let mut is_desktop_entry_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            is_desktop_entry_section = trimmed == "[Desktop Entry]";
        }
        if !is_desktop_entry_section {
            continue;
        }

        if let Some(pos) = trimmed.find('=') {
            let key = trimmed[..pos].trim();
            let value = trimmed[pos + 1..].trim();
            match key {
                "Name" if name.is_none() => {
                    name = Some(value.to_string());
                }
                "Exec" if exec.is_none() => {
                    exec = Some(clean_exec_command(value));
                }
                "Icon" if icon.is_none() => {
                    icon = Some(value.to_string());
                }
                "NoDisplay" if value.to_lowercase() == "true" => {
                    no_display = true;
                }
                _ => {}
            }
        }
    }

    if no_display {
        return None;
    }

    if let (Some(n), Some(e)) = (name, exec) {
        if n.trim().is_empty() || e.trim().is_empty() {
            return None;
        }
        let i = icon.unwrap_or_default();
        Some(DesktopApp { name: n, exec: e, icon: i, path })
    } else {
        None
    }
}

fn split_exec_line(raw: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    
    for c in raw.chars() {
        match c {
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            ' ' | '\t' if !in_double_quote && !in_single_quote => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn clean_exec_command(raw_exec: &str) -> String {
    let tokens = split_exec_line(raw_exec);
    let mut exe_token = "";
    
    for tok in &tokens {
        if tok.starts_with('%') {
            continue;
        }
        if tok.contains('=') {
            continue;
        }
        if tok == "env" {
            continue;
        }
        exe_token = tok;
        break;
    }

    if exe_token.is_empty() {
        return String::new();
    }
    
    // Resolve filename (basename)
    if let Some(pos) = exe_token.rfind('/') {
        exe_token[pos + 1..].to_string()
    } else {
        exe_token.to_string()
    }
}

const PROC_NOISE: &[&str] = &[
    "systemd", "kworker", "dbus-daemon", "Xorg", "Xwayland",
    "pulseaudio", "pipewire", "wireplumber", "gdm", "lightdm",
    "NetworkManager", "wpa_supplicant", "bluetoothd", "udisksd",
    "upowerd", "packagekitd", "polkitd", "rsyslogd", "cron",
];

pub fn scan_running_processes() -> Vec<DesktopApp> {
    let mut seen = std::collections::HashSet::new();
    let mut apps = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            if !fname.to_string_lossy().chars().all(|c| c.is_ascii_digit()) { continue; }
            let exe_link = entry.path().join("exe");
            if let Ok(target) = std::fs::read_link(&exe_link) {
                if let Some(basename) = target.file_name() {
                    let exe = basename.to_string_lossy().to_string();
                    if exe.starts_with('[') { continue; }
                    if PROC_NOISE.contains(&exe.as_str()) { continue; }
                    if seen.insert(exe.clone()) {
                        apps.push(DesktopApp {
                            name: exe.clone(),
                            exec: exe,
                            icon: String::new(),
                            path: String::new(),
                        });
                    }
                }
            }
        }
    }
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_exec_command() {
        assert_eq!(clean_exec_command("brave-browser %U"), "brave-browser");
        assert_eq!(clean_exec_command("/usr/bin/brave-browser-stable %U"), "brave-browser-stable");
        assert_eq!(clean_exec_command("env BAMF_DESKTOP_FILE_HINT=foo /usr/bin/spotify"), "spotify");
        assert_eq!(clean_exec_command("\"/opt/My App/bin/myapp\" --some-arg"), "myapp");
    }
}
