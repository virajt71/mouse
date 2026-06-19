use crate::lock_ext::MutexExt;
use anyhow::Result;
use evdev::{Device, EventType, InputEvent, Key};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::simulator::KeySimulator;
use crate::engine::modifier_state::ModifierState;

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
                            let has_rel = dev
                                .supported_relative_axes()
                                .map(|axes| {
                                    axes.contains(evdev::RelativeAxisType::REL_X)
                                        || axes.contains(evdev::RelativeAxisType::REL_Y)
                                })
                                .unwrap_or(false);

                            let has_keys = dev
                                .supported_keys()
                                .map(|keys| keys.contains(Key::KEY_A) && keys.contains(Key::KEY_F1))
                                .unwrap_or(false);

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
        mappings: Arc<std::sync::RwLock<HashMap<String, std::sync::Arc<str>>>>,
        uinput_device: Arc<std::sync::Mutex<Option<evdev::uinput::VirtualDevice>>>,
        key_simulator: KeySimulator,
        engine: crate::engine::Engine,
        modifier_state: Arc<ModifierState>,
    ) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Open the device synchronously to catch transient race conditions early
        let mut dev = Device::open(&dev_path)?;

        use nix::fcntl::{fcntl, FcntlArg, OFlag};
        use std::os::unix::io::AsRawFd;
        let fd = dev.as_raw_fd();
        if let Ok(flags) = fcntl(fd, FcntlArg::F_GETFL) {
            let mut oflags = OFlag::from_bits_truncate(flags);
            oflags.insert(OFlag::O_NONBLOCK);
            if let Err(e) = fcntl(fd, FcntlArg::F_SETFL(oflags)) {
                return Err(anyhow::anyhow!(
                    "Failed to set non-blocking on {}: {}",
                    dev_path,
                    e
                ));
            }
        } else {
            return Err(anyhow::anyhow!(
                "Failed to get fcntl flags for {}",
                dev_path
            ));
        }

        // Acquire exclusive grab
        if let Err(e) = dev.grab() {
            return Err(anyhow::anyhow!(
                "Failed to grab keyboard {}: {}",
                dev_path,
                e
            ));
        }
        log::info!(
            "[KeyboardHook] Grabbed physical keyboard exclusively: {}",
            dev_path
        );

        self.device_path = Some(dev_path.clone());
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let dev_path_clone = dev_path.clone();

        let handle = thread::Builder::new()
            .name(format!("KeyboardHook-{}", dev_path_clone))
            .spawn(move || {
                use nix::poll::{poll, PollFd, PollFlags, PollTimeout};
                let mut fds = [PollFd::new(unsafe { std::os::fd::BorrowedFd::borrow_raw(fd) }, PollFlags::POLLIN)];

                let mut pressed_keys = std::collections::HashSet::new();
                let mut intercepted_keys = std::collections::HashSet::new();
                let mut shift_pressed = false;
                let mut ctrl_pressed = false;
                let mut alt_pressed = false;
                let mut alt_gr_pressed = false;
                let mut meta_pressed = false;

                let mut active_dead_key: Option<&'static str> = None;
                let mut hangul_composer = super::simulator::hangul::HangulComposer::new();

                while running.load(Ordering::SeqCst) {
                    let timeout = PollTimeout::try_from(Duration::from_millis(200)).unwrap_or(PollTimeout::NONE);
                    match poll(&mut fds, timeout) {
                        Ok(n) if n > 0 => {
                            match dev.fetch_events() {
                                Ok(events) => {
                                    for event in events {
                                        let mut should_forward = true;

                                        let is_forwarding = crate::flow::FLOW_MANAGER.is_forwarding_to_remote();
                                        if is_forwarding {
                                            if let Some(peer_name) = crate::flow::FLOW_MANAGER.get_active_peer_name() {
                                                if event.event_type() == EventType::KEY {
                                                    let evt = crate::flow::network::FlowEvent::Key { code: event.code(), value: event.value() };
                                                    let _ = crate::flow::network::send_event_to_peer(&peer_name, &evt);
                                                }
                                            }
                                            should_forward = false;
                                        } else if event.event_type() == EventType::KEY {
                                            let key_code = event.code();
                                            let down = event.value() != 0;

                                            // Update shared modifier state atomics (OR semantics across hooks).
                                            // Update local booleans too for intra-thread logic below.
                                            if key_code == Key::KEY_LEFTSHIFT.0 || key_code == Key::KEY_RIGHTSHIFT.0 {
                                                shift_pressed = down;
                                                modifier_state.shift.store(down, Ordering::SeqCst);
                                            } else if key_code == Key::KEY_LEFTCTRL.0 || key_code == Key::KEY_RIGHTCTRL.0 {
                                                ctrl_pressed = down;
                                                modifier_state.ctrl.store(down, Ordering::SeqCst);
                                            } else if key_code == Key::KEY_LEFTALT.0 {
                                                alt_pressed = down;
                                                modifier_state.alt.store(down, Ordering::SeqCst);
                                            } else if key_code == Key::KEY_RIGHTALT.0 {
                                                alt_gr_pressed = down;
                                                // AltGr (Right Alt) is not a flow modifier — don't set alt atomic
                                            } else if key_code == Key::KEY_LEFTMETA.0 || key_code == Key::KEY_RIGHTMETA.0 {
                                                meta_pressed = down;
                                                modifier_state.meta.store(down, Ordering::SeqCst);
                                            }

                                            // Reset composition engine on shortcut triggers
                                            if ctrl_pressed || alt_pressed || meta_pressed {
                                                active_dead_key = None;
                                                hangul_composer.reset();
                                            }

                                            if key_code == Key::KEY_KBDILLUMTOGGLE.0 {
                                                if event.value() == 1 {
                                                    log::info!("[KeyboardHook] Captured Fn + Lightbulb (KEY_KBDILLUMTOGGLE). Cycling backlight effect...");
                                                    engine.cycle_backlight_effect();
                                                }
                                                should_forward = false;
                                            } else {
                                                // Language keyboard translation
                                                let layout = key_simulator.get_keyboard_layout();
                                                let is_lang = super::simulator::layout_translator::is_language_layout(&layout);

                                                if is_lang && !ctrl_pressed && !alt_pressed && !meta_pressed {
                                                    if down {
                                                        if layout == "Korean (2-set) Hangul" {
                                                            if key_code == Key::KEY_BACKSPACE.0 {
                                                                hangul_composer.reset();
                                                            } else if let Some(unicode_str) = super::simulator::layout_translator::translate_key(&layout, key_code, shift_pressed, alt_gr_pressed) {
                                                                for ch in unicode_str.chars() {
                                                                    match hangul_composer.feed(ch) {
                                                                        super::simulator::hangul::HangulAction::Update { backspaces, text } => {
                                                                            for _ in 0..backspaces {
                                                                                key_simulator.send_key_combo(&[Key::KEY_BACKSPACE], 5);
                                                                            }
                                                                            for tc in text.chars() {
                                                                                key_simulator.type_unicode_char(tc);
                                                                            }
                                                                        }
                                                                        super::simulator::hangul::HangulAction::ResetAndType(raw_ch) => {
                                                                            key_simulator.type_unicode_char(raw_ch);
                                                                        }
                                                                    }
                                                                }
                                                                intercepted_keys.insert(key_code);
                                                                should_forward = false;
                                                            }
                                                        } else {
                                                            hangul_composer.reset();

                                                            if key_code == Key::KEY_BACKSPACE.0 {
                                                                active_dead_key = None;
                                                            } else if let Some(unicode_str) = super::simulator::layout_translator::translate_key(&layout, key_code, shift_pressed, alt_gr_pressed) {
                                                                if unicode_str.starts_with("dead_") {
                                                                    active_dead_key = Some(unicode_str);
                                                                } else {
                                                                    if let Some(dead_key_name) = active_dead_key.take() {
                                                                        for ch in unicode_str.chars() {
                                                                            if let Some(comp_ch) = super::simulator::compose::compose_char(dead_key_name, ch) {
                                                                                key_simulator.type_unicode_char(comp_ch);
                                                                            } else {
                                                                                let standalone = super::simulator::compose::get_dead_key_standalone(dead_key_name);
                                                                                key_simulator.type_unicode_char(standalone);
                                                                                if ch != ' ' {
                                                                                    key_simulator.type_unicode_char(ch);
                                                                                }
                                                                            }
                                                                        }
                                                                    } else {
                                                                        // Mirror brackets for RTL layouts
                                                                        let is_rtl = super::simulator::layout_translator::is_rtl_layout(&layout);
                                                                        for ch in unicode_str.chars() {
                                                                            let target_ch = if is_rtl {
                                                                                match ch {
                                                                                    '(' => ')',
                                                                                    ')' => '(',
                                                                                    '[' => ']',
                                                                                    ']' => '[',
                                                                                    '{' => '}',
                                                                                    '}' => '{',
                                                                                    '<' => '>',
                                                                                    '>' => '<',
                                                                                    other => other,
                                                                                }
                                                                            } else {
                                                                                ch
                                                                            };
                                                                            key_simulator.type_unicode_char(target_ch);
                                                                        }
                                                                    }
                                                                }
                                                                intercepted_keys.insert(key_code);
                                                                should_forward = false;
                                                            } else {
                                                                // If a key has no translation, flush dead key first if active
                                                                if let Some(dead_key_name) = active_dead_key.take() {
                                                                    let standalone = super::simulator::compose::get_dead_key_standalone(dead_key_name);
                                                                    key_simulator.type_unicode_char(standalone);
                                                                }
                                                            }
                                                        }
                                                    } else if intercepted_keys.remove(&key_code) {
                                                        should_forward = false;
                                                    }
                                                }

                                                if should_forward {
                                                    // Map F1-F12 and other customizable keys
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
                                                        k if k == Key::KEY_INSERT.0 => Some("insert"),
                                                        k if k == Key::KEY_HOME.0 => Some("home"),
                                                        k if k == Key::KEY_PAGEUP.0 => Some("pageup"),
                                                        k if k == Key::KEY_DELETE.0 => Some("delete"),
                                                        k if k == Key::KEY_END.0 => Some("end"),
                                                        k if k == Key::KEY_PAGEDOWN.0 => Some("pagedown"),
                                                        k if k == Key::KEY_CALC.0 => Some("calculator"),
                                                        k if k == Key::KEY_COFFEE.0 => Some("screenlock"),
                                                        k if k == Key::KEY_SEARCH.0 => Some("search"),
                                                        k if k == Key::KEY_SUSPEND.0 || k == Key::KEY_SLEEP.0 => Some("lockpower"),
                                                        _ => None,
                                                    };

                                                    if let Some(name) = key_name {
                                                        let mapping = {
                                                            mappings.read().unwrap().get(name).cloned()
                                                        };

                                                        if let Some(action_id) = mapping {
                                                            if &*action_id != "none" {
                                                                if down {
                                                                    key_simulator.execute_action(&action_id);
                                                                }
                                                                should_forward = false;
                                                            }
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
                                        }

                                        if should_forward {
                                            if let Some(uinput_lock) = uinput_device.lock_safe().as_mut() {
                                                let _ = uinput_lock.emit(&[event]);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    if e.kind() != std::io::ErrorKind::WouldBlock {
                                        if e.raw_os_error() == Some(19) {
                                            log::info!("[KeyboardHook] Physical keyboard on {} unplugged/disconnected.", dev_path_clone);
                                        } else {
                                            log::warn!("[KeyboardHook] Read error on {}: {}. Releasing grab.", dev_path_clone, e);
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Ok(_) => {} // Timeout reached
                        Err(e) => {
                            if e != nix::errno::Errno::EINTR {
                                if e == nix::errno::Errno::ENODEV {
                                    log::info!("[KeyboardHook] Physical keyboard on {} unplugged/disconnected.", dev_path_clone);
                                } else {
                                    log::error!("[KeyboardHook] Poll error on {}: {}. Releasing grab.", dev_path_clone, e);
                                }
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
                log::info!("[KeyboardHook] Ungrabbed physical keyboard: {}", dev_path_clone);
                running.store(false, Ordering::SeqCst);
            })
            .expect("Failed to spawn KeyboardHook thread");

        self.thread_handle = Some(handle);
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn device_path(&self) -> Option<&str> {
        self.device_path.as_deref()
    }

    pub fn stop(&mut self) {
        log::info!(
            "[KeyboardHook] Stopping listener for {:?}...",
            self.device_path
        );
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
