use anyhow::Result;
use evdev::{Device, EventType, Key, InputEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

pub struct KeyboardHook {
    device_path: Option<String>,
    running: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

pub fn find_logitech_keyboards() -> Vec<String> {
    let mut list = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with("event") {
                    if let Ok(dev) = Device::open(&path) {
                        let id = dev.input_id();
                        if id.vendor() == 0x046D {
                            // A keyboard should have many keys but NOT relative axes (X/Y)
                            let has_rel = dev.supported_relative_axes().map(|axes| {
                                axes.contains(evdev::RelativeAxisType::REL_X) || axes.contains(evdev::RelativeAxisType::REL_Y)
                            }).unwrap_or(false);

                            let has_keys = dev.supported_keys().map(|keys| {
                                keys.contains(Key::KEY_A) && keys.contains(Key::KEY_F1)
                            }).unwrap_or(false);

                            if has_keys && !has_rel {
                                list.push(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    list
}

impl Default for KeyboardHook {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardHook {
    pub fn new() -> Self {
        KeyboardHook {
            device_path: None,
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    pub fn start(
        &mut self,
        dev_path: String,
        mappings: Arc<std::sync::Mutex<HashMap<String, String>>>,
        uinput_device: Arc<std::sync::Mutex<Option<evdev::uinput::VirtualDevice>>>,
        key_simulator: crate::key_simulator::KeySimulator,
    ) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.device_path = Some(dev_path.clone());
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        let handle = thread::Builder::new()
            .name(format!("KeyboardHook-{}", dev_path))
            .spawn(move || {
                let mut dev = match Device::open(&dev_path) {
                    Ok(d) => d,
                    Err(e) => {
                        log::error!("[KeyboardHook] Failed to open {}: {}", dev_path, e);
                        running.store(false, Ordering::SeqCst);
                        return;
                    }
                };

                use std::os::unix::io::AsRawFd;
                use nix::fcntl::{fcntl, FcntlArg, OFlag};
                let fd = dev.as_raw_fd();
                if let Ok(flags) = fcntl(fd, FcntlArg::F_GETFL) {
                    let mut oflags = OFlag::from_bits_truncate(flags);
                    oflags.insert(OFlag::O_NONBLOCK);
                    if let Err(e) = fcntl(fd, FcntlArg::F_SETFL(oflags)) {
                        log::error!("[KeyboardHook] Failed to set non-blocking on {}: {}", dev_path, e);
                        running.store(false, Ordering::SeqCst);
                        return;
                    }
                } else {
                    log::error!("[KeyboardHook] Failed to get fcntl flags for {}", dev_path);
                    running.store(false, Ordering::SeqCst);
                    return;
                }

                // Acquire exclusive grab
                if let Err(e) = dev.grab() {
                    log::error!("[KeyboardHook] Failed to grab keyboard {}: {}", dev_path, e);
                    running.store(false, Ordering::SeqCst);
                    return;
                }
                log::info!("[KeyboardHook] Grabbed physical keyboard exclusively: {}", dev_path);

                use nix::poll::{poll, PollFd, PollFlags, PollTimeout};
                let mut fds = [PollFd::new(unsafe { std::os::fd::BorrowedFd::borrow_raw(fd) }, PollFlags::POLLIN)];

                let mut pressed_keys = std::collections::HashSet::new();

                while running.load(Ordering::SeqCst) {
                    let timeout = PollTimeout::try_from(Duration::from_millis(200)).unwrap_or(PollTimeout::NONE);
                    match poll(&mut fds, timeout) {
                        Ok(n) if n > 0 => {
                            match dev.fetch_events() {
                                Ok(events) => {
                                    for event in events {
                                        let mut should_forward = true;

                                        if event.event_type() == EventType::KEY {
                                            let key_code = event.code();
                                            let down = event.value() != 0;

                                            // Map F1-F12 keys
                                            let key_name = match key_code {
                                                k if k == Key::KEY_F1.0 => Some("f1"),
                                                k if k == Key::KEY_F2.0 => Some("f2"),
                                                k if k == Key::KEY_F3.0 => Some("f3"),
                                                k if k == Key::KEY_F4.0 => Some("f4"),
                                                k if k == Key::KEY_F5.0 => Some("f5"),
                                                k if k == Key::KEY_F6.0 => Some("f6"),
                                                k if k == Key::KEY_F7.0 => Some("f7"),
                                                k if k == Key::KEY_F8.0 => Some("f8"),
                                                k if k == Key::KEY_F9.0 => Some("f9"),
                                                k if k == Key::KEY_F10.0 => Some("f10"),
                                                k if k == Key::KEY_F11.0 => Some("f11"),
                                                k if k == Key::KEY_F12.0 => Some("f12"),
                                                _ => None,
                                            };

                                            if let Some(name) = key_name {
                                                let mapping = {
                                                    mappings.lock().unwrap().get(name).cloned()
                                                };

                                                if let Some(action_id) = mapping {
                                                    if action_id != "none" {
                                                        if down {
                                                            key_simulator.execute_action(&action_id);
                                                        }
                                                        should_forward = false;
                                                    }
                                                }
                                            }

                                            if should_forward {
                                                if down {
                                                    pressed_keys.insert(key_code);
                                                } else {
                                                    pressed_keys.remove(&key_code);
                                                }
                                            }
                                        }

                                        if should_forward {
                                            if let Some(uinput_lock) = uinput_device.lock().unwrap().as_mut() {
                                                let _ = uinput_lock.emit(&[event]);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    if e.kind() != std::io::ErrorKind::WouldBlock {
                                        if e.raw_os_error() == Some(19) {
                                            log::info!("[KeyboardHook] Physical keyboard on {} unplugged/disconnected.", dev_path);
                                        } else {
                                            log::warn!("[KeyboardHook] Read error on {}: {}. Releasing grab.", dev_path, e);
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Ok(_) => {} // Timeout reached
                        Err(e) => {
                            if e != nix::errno::Errno::EINTR {
                                log::error!("[KeyboardHook] Poll error on {}: {}. Releasing grab.", dev_path, e);
                                break;
                            }
                        }
                    }
                }

                // Release any stuck keys on the virtual uinput device
                if !pressed_keys.is_empty() {
                    log::info!("[KeyboardHook] Releasing {} stuck keys on uinput during shutdown", pressed_keys.len());
                    if let Ok(mut uinput_lock) = uinput_device.lock() {
                        if let Some(uinput_dev) = uinput_lock.as_mut() {
                            let mut release_events = Vec::new();
                            for &key_code in &pressed_keys {
                                release_events.push(InputEvent::new(EventType::KEY, key_code, 0));
                            }
                            release_events.push(InputEvent::new(EventType::SYNCHRONIZATION, 0, 0));
                            let _ = uinput_dev.emit(&release_events);
                        }
                    }
                }

                let _ = dev.ungrab();
                log::info!("[KeyboardHook] Ungrabbed physical keyboard: {}", dev_path);
                running.store(false, Ordering::SeqCst);
            })
            .expect("Failed to spawn KeyboardHook thread");

        self.thread_handle = Some(handle);
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn stop(&mut self) {
        log::info!("[KeyboardHook] Stopping listener for {:?}...", self.device_path);
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
