use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use mouser_engine::config::{execs_match, normalize_exec};

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
                        if let Some(app) =
                            parse_desktop_file(&content, path.to_string_lossy().into_owned())
                        {
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
        Some(DesktopApp {
            name: n,
            exec: e,
            icon: i,
            path,
        })
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
    "systemd",
    "kworker",
    "dbus-daemon",
    "Xorg",
    "Xwayland",
    "pulseaudio",
    "pipewire",
    "wireplumber",
    "gdm",
    "lightdm",
    "NetworkManager",
    "wpa_supplicant",
    "bluetoothd",
    "udisksd",
    "upowerd",
    "packagekitd",
    "polkitd",
    "rsyslogd",
    "cron",
];

pub fn scan_running_processes() -> Vec<DesktopApp> {
    let mut seen = std::collections::HashSet::new();
    let mut apps = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            if !fname.to_string_lossy().chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let exe_link = entry.path().join("exe");
            if let Ok(target) = std::fs::read_link(&exe_link) {
                if let Some(basename) = target.file_name() {
                    let exe = basename.to_string_lossy().to_string();
                    if exe.starts_with('[') {
                        continue;
                    }
                    if PROC_NOISE.contains(&exe.as_str()) {
                        continue;
                    }
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

// --- execs_match and normalize_exec imported from mouser_engine::config ---

/// Strict exec comparison — normalize + exact equality only, no substring
/// containment. Used for deduplicating installed desktop files against each
/// other, where "antigravity" and "antigravity-ide" must remain separate.
fn execs_strict(a: &str, b: &str) -> bool {
    let na = normalize_exec(a);
    let nb = normalize_exec(b);
    !na.is_empty() && !nb.is_empty() && na == nb
}

pub fn scan_all_applications() -> Vec<DesktopApp> {
    let installed = scan_desktop_applications();
    let running = scan_running_processes();

    let mut combined: Vec<DesktopApp> = Vec::new();

    // 1. Add all installed applications first. Use *strict* matching so that
    // unrelated apps with a common prefix (e.g. "antigravity" vs
    // "antigravity-ide") stay separate.
    for app in installed {
        if !app.exec.is_empty()
            && !combined
                .iter()
                .any(|existing| execs_strict(&existing.exec, &app.exec))
        {
            combined.push(app);
        }
    }

    // 2. Add running applications only if no installed app already matches
    // them (fuzzy, with substring containment) — a wrapper/binary name
    // mismatch (e.g. brave-browser-stable vs brave) creates a second,
    // icon-less duplicate when exact matching alone is used.
    for app in running {
        if !app.exec.is_empty()
            && !combined
                .iter()
                .any(|existing| execs_match(&existing.exec, &app.exec))
        {
            combined.push(app);
        }
    }

    // Sort alphabetically by name
    combined.sort_by_key(|a| a.name.to_lowercase());
    combined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_exec_command() {
        assert_eq!(clean_exec_command("brave-browser %U"), "brave-browser");
        assert_eq!(
            clean_exec_command("/usr/bin/brave-browser-stable %U"),
            "brave-browser-stable"
        );
        assert_eq!(
            clean_exec_command("env BAMF_DESKTOP_FILE_HINT=foo /usr/bin/spotify"),
            "spotify"
        );
        assert_eq!(
            clean_exec_command("\"/opt/My App/bin/myapp\" --some-arg"),
            "myapp"
        );
    }

    #[test]
    fn test_execs_match_handles_channel_suffix_and_wrapper_mismatch() {
        // .desktop Exec vs the real /proc/*/exe basename for the same app.
        assert!(execs_match("brave-browser-stable", "brave"));
        assert!(execs_match("brave", "brave-browser-stable"));
        assert!(!execs_match("brave", "firefox"));
        assert!(!execs_match("", "brave"));
    }

    #[test]
    fn test_scan_all_applications_dedupes_desktop_and_process_variants() {
        let installed = vec![DesktopApp {
            name: "Brave Web Browser".into(),
            exec: "brave-browser-stable".into(),
            icon: "brave-browser".into(),
            path: "/usr/share/applications/brave-browser.desktop".into(),
        }];
        let running = vec![DesktopApp {
            name: "brave".into(),
            exec: "brave".into(),
            icon: String::new(),
            path: String::new(),
        }];

        let mut combined: Vec<DesktopApp> = Vec::new();
        for app in installed {
            if !app.exec.is_empty()
                && !combined
                    .iter()
                    .any(|e: &DesktopApp| execs_match(&e.exec, &app.exec))
            {
                combined.push(app);
            }
        }
        for app in running {
            if !app.exec.is_empty()
                && !combined
                    .iter()
                    .any(|e: &DesktopApp| execs_match(&e.exec, &app.exec))
            {
                combined.push(app);
            }
        }

        assert_eq!(
            combined.len(),
            1,
            "Brave should appear once, with its real icon kept"
        );
        assert_eq!(combined[0].icon, "brave-browser");
    }

    #[test]
    fn test_antigravity_and_antigravity_ide_stay_separate() {
        let apps = vec![
            DesktopApp {
                name: "Antigravity".into(),
                exec: "antigravity".into(),
                icon: "antigravity".into(),
                path: "antigravity.desktop".into(),
            },
            DesktopApp {
                name: "Antigravity IDE".into(),
                exec: "antigravity-ide".into(),
                icon: "antigravity-ide".into(),
                path: "antigravity-ide.desktop".into(),
            },
        ];

        let mut combined: Vec<DesktopApp> = Vec::new();
        for app in apps {
            if !app.exec.is_empty()
                && !combined
                    .iter()
                    .any(|e: &DesktopApp| execs_strict(&e.exec, &app.exec))
            {
                combined.push(app);
            }
        }

        assert_eq!(
            combined.len(),
            2,
            "antigravity and antigravity-ide should be separate entries"
        );
    }

    #[test]
    fn test_scan_all_applications() {
        let apps = scan_all_applications();
        println!("Found {} applications.", apps.len());
    }

    #[test]
    fn test_real_world_no_duplicates() {
        let apps = scan_all_applications();
        // Check that the two brave .desktop files don't create duplicates
        // via name-dedup in scan_desktop_applications + execs_match in scan_all_applications
        let brave_entries: Vec<_> = apps
            .iter()
            .filter(|a| a.name.to_lowercase().contains("brave"))
            .collect();
        if brave_entries.len() > 1 {
            println!("Found {} brave entries (DUPLICATE!):", brave_entries.len());
            for a in brave_entries {
                println!("  name={}, exec={}, icon={:?}", a.name, a.exec, a.icon);
            }
            panic!("Brave appears more than once in scanned apps");
        } else if brave_entries.len() == 1 {
            println!(
                "OK: single brave entry: name={}, exec={}, icon={:?}",
                brave_entries[0].name, brave_entries[0].exec, brave_entries[0].icon
            );
        } else {
            println!("Brave not found on this system, test skipped");
        }
    }
}
