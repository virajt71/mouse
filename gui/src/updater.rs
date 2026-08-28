use eframe::egui;
use mouser_engine::lock_ext::MutexExt;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    Available {
        version: String,
        bin_url: String,
        sha_url: Option<String>,
        bin_name: String,
    },
    Downloading {
        progress: f32, // 0.0 to 1.0
    },
    Verifying,
    ReadyToInstall {
        version: String,
        local_path: PathBuf,
    },
    Installing,
    RestartRequired,
    Failed(String),
    UpToDate,
}

pub struct Updater {
    pub status: Arc<Mutex<UpdateStatus>>,
}

impl Default for Updater {
    fn default() -> Self {
        Self {
            status: Arc::new(Mutex::new(UpdateStatus::Idle)),
        }
    }
}

#[derive(serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(serde::Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

impl Updater {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawns a background thread to check for updates.
    pub fn check_for_updates(&self, ctx: egui::Context, auto_install: bool) {
        let status_clone = self.status.clone();

        {
            let mut status = status_clone.lock_safe();
            *status = UpdateStatus::Checking;
        }
        ctx.request_repaint();

        std::thread::spawn(move || {
            // Check for mock update environment variable
            if std::env::var("MOUSER_MOCK_UPDATE").is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(1500));
                let mock_version = "0.2.0".to_string();
                let mock_bin_url = "https://github.com/soulr27/Mouser-RS-UI/releases/download/v0.2.0/mouser-rs-linux".to_string();

                {
                    let mut status = status_clone.lock_safe();
                    *status = UpdateStatus::Available {
                        version: mock_version.clone(),
                        bin_url: mock_bin_url.clone(),
                        sha_url: None,
                        bin_name: "mouser-rs-linux".to_string(),
                    };
                }
                ctx.request_repaint();

                if auto_install {
                    Self::download_and_install_mock(ctx, status_clone, mock_version);
                }
                return;
            }

            // Real GitHub Releases API check
            match Self::fetch_latest_release() {
                Ok(release) => {
                    let current_version = env!("CARGO_PKG_VERSION");
                    if is_newer_version(&release.tag_name, current_version) {
                        // Find linux binary and checksum assets
                        let mut bin_asset = None;
                        let mut sha_asset = None;

                        for asset in &release.assets {
                            let name_lower = asset.name.to_lowercase();
                            if name_lower.contains("sha256") || name_lower.contains("checksum") {
                                sha_asset = Some(asset.browser_download_url.clone());
                            } else if name_lower.contains("linux") || name_lower == "mouser-rs" {
                                bin_asset =
                                    Some((asset.name.clone(), asset.browser_download_url.clone()));
                            }
                        }

                        if let Some((bin_name, bin_url)) = bin_asset {
                            let version_str = release.tag_name.clone();
                            {
                                let mut status = status_clone.lock_safe();
                                *status = UpdateStatus::Available {
                                    version: version_str.clone(),
                                    bin_url: bin_url.clone(),
                                    sha_url: sha_asset.clone(),
                                    bin_name: bin_name.clone(),
                                };
                            }
                            ctx.request_repaint();

                            if auto_install {
                                let ctx2 = ctx.clone();
                                Self::download_and_install_real(
                                    ctx2,
                                    status_clone,
                                    version_str,
                                    bin_url,
                                    sha_asset,
                                    bin_name,
                                );
                            }
                        } else {
                            let mut status = status_clone.lock_safe();
                            *status = UpdateStatus::UpToDate;
                        }
                    } else {
                        let mut status = status_clone.lock_safe();
                        *status = UpdateStatus::UpToDate;
                    }
                }
                Err(err) => {
                    let mut status = status_clone.lock_safe();
                    if auto_install {
                        *status = UpdateStatus::Idle;
                    } else {
                        *status = UpdateStatus::Failed(format!("Check failed: {}", err));
                    }
                }
            }
            ctx.request_repaint();
        });
    }

    /// Spawns background thread to download and install the update.
    pub fn start_download_and_install(
        &self,
        ctx: egui::Context,
        version: String,
        bin_url: String,
        sha_url: Option<String>,
        bin_name: String,
    ) {
        let status_clone = self.status.clone();

        std::thread::spawn(move || {
            if std::env::var("MOUSER_MOCK_UPDATE").is_ok() {
                Self::download_and_install_mock(ctx, status_clone, version);
            } else {
                Self::download_and_install_real(
                    ctx,
                    status_clone,
                    version,
                    bin_url,
                    sha_url,
                    bin_name,
                );
            }
        });
    }

    fn fetch_latest_release() -> Result<GitHubRelease, String> {
        let response =
            ureq::get("https://api.github.com/repos/soulr27/Mouser-RS-UI/releases/latest")
                .timeout(std::time::Duration::from_secs(10))
                .set("User-Agent", "mouser-rs-updater")
                .call()
                .map_err(|e| e.to_string())?;

        let release: GitHubRelease = response.into_json().map_err(|e| e.to_string())?;
        Ok(release)
    }

    fn download_and_install_mock(
        ctx: egui::Context,
        status_ref: Arc<Mutex<UpdateStatus>>,
        version: String,
    ) {
        // Simulate download progress
        for i in 0..=10 {
            {
                let mut status = status_ref.lock_safe();
                *status = UpdateStatus::Downloading {
                    progress: i as f32 / 10.0,
                };
            }
            ctx.request_repaint();
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        {
            let mut status = status_ref.lock_safe();
            *status = UpdateStatus::Verifying;
        }
        ctx.request_repaint();
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Create mock binary at local path (just copy current running exe to represent new file)
        let Some(data_dir) = dirs::data_local_dir() else {
            let mut status = status_ref.lock_safe();
            *status = UpdateStatus::Failed("Couldn't find a place to save the update. Check your disk permissions and try again.".to_string());
            ctx.request_repaint();
            return;
        };
        let mouser_dir = data_dir.join("mouser-rs");
        let _ = std::fs::create_dir_all(&mouser_dir);
        let mock_temp_bin = mouser_dir.join("mouser-rs.tmp");

        if let Ok(current_exe) = std::env::current_exe() {
            let _ = std::fs::copy(&current_exe, &mock_temp_bin);
        } else {
            let _ = std::fs::write(&mock_temp_bin, b"mock binary data");
        }

        {
            let mut status = status_ref.lock_safe();
            *status = UpdateStatus::ReadyToInstall {
                version: version.clone(),
                local_path: mock_temp_bin,
            };
        }
        ctx.request_repaint();

        // Perform mock replacement
        std::thread::sleep(std::time::Duration::from_millis(500));
        {
            let mut status = status_ref.lock_safe();
            *status = UpdateStatus::Installing;
        }
        ctx.request_repaint();

        let mut status = status_ref.lock_safe();
        // Since we are mocking, we do not overwrite the active binary to prevent breaking the editor's execution flow,
        // but we simulate that it completed successfully and requires restart.
        *status = UpdateStatus::RestartRequired;
        ctx.request_repaint();
    }

    fn download_and_install_real(
        ctx: egui::Context,
        status_ref: Arc<Mutex<UpdateStatus>>,
        _version: String,
        bin_url: String,
        sha_url: Option<String>,
        bin_name: String,
    ) {
        {
            let mut status = status_ref.lock_safe();
            *status = UpdateStatus::Downloading { progress: 0.0 };
        }
        ctx.request_repaint();

        let Some(data_dir) = dirs::data_local_dir() else {
            let mut status = status_ref.lock_safe();
            *status = UpdateStatus::Failed("Couldn't find a place to save the update. Check your disk permissions and try again.".to_string());
            ctx.request_repaint();
            return;
        };
        let mouser_dir = data_dir.join("mouser-rs");
        let _ = std::fs::create_dir_all(&mouser_dir);
        let temp_bin = mouser_dir.join("mouser-rs.tmp");

        // 1. Download binary asset
        match download_file(&bin_url, &temp_bin, &ctx, &status_ref) {
            Ok(_) => {
                {
                    let mut status = status_ref.lock_safe();
                    *status = UpdateStatus::Verifying;
                }
                ctx.request_repaint();

                // 2. Download and verify Checksum if present
                if let Some(sha_url_str) = sha_url {
                    match verify_checksum(&sha_url_str, &temp_bin, &bin_name) {
                        Ok(true) => {}
                        Ok(false) => {
                            let mut status = status_ref.lock_safe();
                            *status = UpdateStatus::Failed(
                                "Update file failed a security check and won't be installed. Try downloading again.".to_string(),
                            );
                            ctx.request_repaint();
                            return;
                        }
                        Err(e) => {
                            let mut status = status_ref.lock_safe();
                            *status = UpdateStatus::Failed(format!("Checksum error: {}", e));
                            ctx.request_repaint();
                            return;
                        }
                    }
                }

                {
                    let mut status = status_ref.lock_safe();
                    *status = UpdateStatus::Installing;
                }
                ctx.request_repaint();

                // 3. Swap the active binary
                match replace_binary(&temp_bin) {
                    Ok(_) => {
                        let mut status = status_ref.lock_safe();
                        *status = UpdateStatus::RestartRequired;
                    }
                    Err(e) => {
                        let mut status = status_ref.lock_safe();
                        *status = UpdateStatus::Failed(format!("Install failed: {}", e));
                    }
                }
                ctx.request_repaint();
            }
            Err(e) => {
                let mut status = status_ref.lock_safe();
                *status = UpdateStatus::Failed(format!("Download failed: {}", e));
                ctx.request_repaint();
            }
        }
    }

    /// Perform hot-swapping of the binary on disk and trigger restart.
    pub fn restart_and_apply() -> Result<(), String> {
        let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;

        // Spawn the replacement process
        std::process::Command::new(current_exe)
            .spawn()
            .map_err(|e| e.to_string())?;

        std::process::exit(0);
    }
}

fn download_file(
    url: &str,
    dest: &Path,
    ctx: &egui::Context,
    status_ref: &Arc<Mutex<UpdateStatus>>,
) -> Result<(), String> {
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| e.to_string())?;

    let content_len = response
        .header("Content-Length")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let mut reader = response.into_reader();
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    let mut buffer = vec![0; 16384];
    let mut downloaded = 0;

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])
            .map_err(|e| e.to_string())?;
        downloaded += bytes_read;

        if content_len > 0 {
            let progress = downloaded as f32 / content_len as f32;
            let mut status = status_ref.lock_safe();
            *status = UpdateStatus::Downloading { progress };
            ctx.request_repaint();
        }
    }
    Ok(())
}

fn verify_checksum(sha_url: &str, bin_path: &Path, bin_name: &str) -> Result<bool, String> {
    let response_str = ureq::get(sha_url)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())?;

    let mut expected_hex = None;
    for line in response_str.lines() {
        if line.contains(bin_name) {
            if let Some(hex) = line.split_whitespace().next() {
                expected_hex = Some(hex.to_string());
                break;
            }
        }
    }

    let Some(hex_str) = expected_hex else {
        return Err(format!(
            "Could not find hash entries matching '{}' in SHA256SUMS file",
            bin_name
        ));
    };

    let mut file = std::fs::File::open(bin_path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
    let hash = hasher.finalize();
    let computed_hex = format!("{:x}", hash);

    Ok(computed_hex.trim().eq_ignore_ascii_case(hex_str.trim()))
}

fn replace_binary(new_bin: &Path) -> Result<(), String> {
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let backup_exe = current_exe.with_extension("old");

    // SWAP: Rename running binary to .old (always allowed on Linux)
    let _ = std::fs::remove_file(&backup_exe); // remove stale backup if present
    std::fs::rename(&current_exe, &backup_exe).map_err(|e| e.to_string())?;

    // Copy new executable to original location
    if let Err(err) = std::fs::copy(new_bin, &current_exe) {
        // Rollback on failure
        let _ = std::fs::rename(&backup_exe, &current_exe);
        return Err(format!("Failed to copy new binary to target path: {}", err));
    }

    // Set execution permissions on Linux
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&current_exe) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&current_exe, perms);
        }
    }

    // Clean up temporary files
    let _ = std::fs::remove_file(new_bin);
    let _ = std::fs::remove_file(backup_exe); // remove old binary

    Ok(())
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let strip = |v: &str| v.trim_start_matches('v').trim().to_string();
    let latest_clean = strip(latest);
    let current_clean = strip(current);

    let parse_parts = |v: &str| -> Vec<u32> {
        v.split('.')
            .map(|s| s.parse::<u32>().unwrap_or(0))
            .collect()
    };

    let latest_parts = parse_parts(&latest_clean);
    let current_parts = parse_parts(&current_clean);

    for i in 0..std::cmp::max(latest_parts.len(), current_parts.len()) {
        let l_val = latest_parts.get(i).cloned().unwrap_or(0);
        let c_val = current_parts.get(i).cloned().unwrap_or(0);
        if l_val > c_val {
            return true;
        } else if l_val < c_val {
            return false;
        }
    }
    false
}
