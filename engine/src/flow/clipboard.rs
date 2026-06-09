use std::sync::Mutex;
use std::time::Duration;
use std::fs;
use std::path::Path;
use arboard::{Clipboard, ImageData};
use super::network::{send_event_to_peer, FlowEvent};
use super::FLOW_MANAGER;

lazy_static::lazy_static! {
    static ref LAST_TEXT: Mutex<String> = Mutex::new(String::new());
}

pub fn run_clipboard_loop() {
    log::info!("[Flow Clipboard] Clipboard monitoring thread running...");

    let mut clipboard_opt = None;

    loop {
        if clipboard_opt.is_none() {
            match Clipboard::new() {
                Ok(c) => {
                    log::info!("[Flow Clipboard] Clipboard initialized successfully.");
                    clipboard_opt = Some(c);
                }
                Err(e) => {
                    log::warn!("[Flow Clipboard] Failed to initialize clipboard: {}. Retrying in 5s...", e);
                    std::thread::sleep(Duration::from_secs(5));
                    continue;
                }
            }
        }

        let clipboard = clipboard_opt.as_mut().unwrap();

        // Only monitor and send if local input has focus (i.e. active_peer is None)
        let active_peer = FLOW_MANAGER.get_active_peer_name();
        if active_peer.is_none() {
            match clipboard.get_text() {
                Ok(text) => {
                    let mut last = LAST_TEXT.lock().unwrap();
                    if text != *last && !text.is_empty() {
                        *last = text.clone();

                        log::debug!("[Flow Clipboard] Local clipboard changed: text len={}", text.len());

                        // If text starts with file://, check if we should stream it as files
                        let event = if text.starts_with("file://") {
                            // Parse local file path
                            let clean_path = text.trim_start_matches("file://").trim();
                            let path = Path::new(clean_path);
                            if path.is_file() {
                                if let Ok(content) = fs::read(path) {
                                    let filename = path.file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string();
                                    Some(FlowEvent::FileTransfer { name: filename, content })
                                } else {
                                    Some(FlowEvent::ClipboardText(text))
                                }
                            } else {
                                Some(FlowEvent::ClipboardText(text))
                            }
                        } else {
                            Some(FlowEvent::ClipboardText(text))
                        };

                        if let Some(evt) = event {
                            // Replicate to all active peer connections
                            let conns = super::network::ACTIVE_CONNECTIONS.read().unwrap();
                            for peer_name in conns.keys() {
                                let _ = send_event_to_peer(peer_name, &evt);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::debug!("[Flow Clipboard] Failed to read clipboard: {}. Resetting context...", e);
                    clipboard_opt = None;
                }
            }
        }

        std::thread::sleep(Duration::from_millis(1000));
    }
}

pub fn set_local_clipboard_text(text: String) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new()?;
    *LAST_TEXT.lock().unwrap() = text.clone();
    clipboard.set_text(text)?;
    log::info!("[Flow Clipboard] Clipboard text synchronized from remote.");
    Ok(())
}

pub fn set_local_clipboard_image(png_bytes: Vec<u8>) -> anyhow::Result<()> {
    // Decode image using image crate (already in Cargo.toml dependencies!)
    if let Ok(img) = image::load_from_memory(&png_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let mut clipboard = Clipboard::new()?;
        let img_data = ImageData {
            width: width as usize,
            height: height as usize,
            bytes: std::borrow::Cow::Owned(rgba.into_raw()),
        };
        clipboard.set_image(img_data)?;
        log::info!("[Flow Clipboard] Clipboard image synchronized from remote.");
    }
    Ok(())
}

pub fn save_flow_file(name: String, content: Vec<u8>) -> anyhow::Result<()> {
    let dir = Path::new("/tmp/mouser_flow");
    fs::create_dir_all(dir)?;
    let file_path = dir.join(&name);
    fs::write(&file_path, content)?;

    // Set clipboard to uri-list of temporary file
    let file_uri = format!("file://{}", file_path.to_string_lossy());
    set_local_clipboard_text(file_uri)?;
    log::info!("[Flow Clipboard] File '{}' received and set to clipboard.", name);
    Ok(())
}
