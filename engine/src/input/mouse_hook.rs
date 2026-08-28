use crate::lock_ext::MutexExt;
use anyhow::{anyhow, Result};
use evdev::{AbsoluteAxisType, Device, EventType, InputEvent, Key, PropType, RelativeAxisType};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum MouseHookEvent {
    Button { key: Key, down: bool },
    Scroll { horizontal: bool, delta: i32 },
    Relative { dx: i32, dy: i32 },
}

pub struct MouseHook {
    device_path: Option<String>,
    running: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
    on_event: Arc<dyn Fn(MouseHookEvent) + Send + Sync + 'static>,
}

pub fn is_hookable_mouse(dev: &Device) -> bool {
    // Exclude virtual devices we created (Mouser Virtual Keyboard)
    if let Some(name) = dev.name() {
        if name.starts_with("Mouser ") {
            return false;
        }
    }

    // A mouse clicks and moves relatively; nothing else qualifies.
    let has_rel = dev
        .supported_relative_axes()
        .map(|axes| {
            axes.contains(RelativeAxisType::REL_X) && axes.contains(RelativeAxisType::REL_Y)
        })
        .unwrap_or(false);

    let has_left = dev
        .supported_keys()
        .map(|keys| keys.contains(Key::BTN_LEFT))
        .unwrap_or(false);

    if !has_rel || !has_left {
        return false;
    }

    // Exclude touchpads, trackpads, touchscreens
    let touches = dev
        .supported_keys()
        .map(|keys| keys.contains(Key::BTN_TOUCH) || keys.contains(Key::BTN_TOOL_FINGER))
        .unwrap_or(false)
        || dev
            .supported_absolute_axes()
            .map(|axes| axes.contains(AbsoluteAxisType::ABS_MT_POSITION_X))
            .unwrap_or(false)
        || dev.properties().contains(PropType::BUTTONPAD)
        || dev.properties().contains(PropType::SEMI_MT)
        || dev.properties().contains(PropType::DIRECT);

    if touches {
        return false;
    }

    // Exclude pointing sticks (TrackPoints)
    if dev.properties().contains(PropType::POINTING_STICK) {
        return false;
    }

    true
}

pub fn find_logitech_mice() -> Vec<String> {
    let mut mice = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with("event") {
                    if let Ok(dev) = Device::open(&path) {
                        let id = dev.input_id();
                        if id.vendor() == 0x046D && is_hookable_mouse(&dev) {
                            mice.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    mice
}

pub fn find_logitech_mouse() -> Option<String> {
    find_logitech_mice().first().cloned()
}

impl MouseHook {
    pub fn new<F>(on_event: F) -> Self
    where
        F: Fn(MouseHookEvent) + Send + Sync + 'static,
    {
        MouseHook {
            device_path: None,
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            on_event: Arc::new(on_event),
        }
    }

    pub fn device_path(&self) -> Option<&str> {
        self.device_path.as_deref()
    }

    pub fn start(
        &mut self,
        dev_path: String,
        blocked_buttons: Arc<std::sync::Mutex<Vec<Key>>>,
        invert_vscroll: Arc<AtomicBool>,
        invert_hscroll: Arc<AtomicBool>,
        block_hscroll: Arc<AtomicBool>,
        gesture_active: Arc<AtomicBool>,
        uinput_device: Arc<std::sync::Mutex<Option<evdev::uinput::VirtualDevice>>>,
    ) -> Result<()> {
        if self.running.load(Ordering::Acquire) {
            return Ok(());
        }

        self.device_path = Some(dev_path.clone());

        let mut dev =
            Device::open(&dev_path).map_err(|e| anyhow!("Failed to open {}: {}", dev_path, e))?;

        use nix::fcntl::{fcntl, FcntlArg, OFlag};
        use std::os::unix::io::AsRawFd;
        let fd = dev.as_raw_fd();
        if let Ok(flags) = fcntl(fd, FcntlArg::F_GETFL) {
            let mut oflags = OFlag::from_bits_truncate(flags);
            oflags.insert(OFlag::O_NONBLOCK);
            if let Err(e) = fcntl(fd, FcntlArg::F_SETFL(oflags)) {
                return Err(anyhow!("Failed to set non-blocking on {}: {}", dev_path, e));
            }
        } else {
            return Err(anyhow!("Failed to get fcntl flags for {}", dev_path));
        }

        // Acquire exclusive grab
        dev.grab().map_err(|e| {
            anyhow!(
                "Failed to grab {}: {}. remap will be disabled. Run as root or adjust permissions.",
                dev_path,
                e
            )
        })?;
        log::info!(
            "[MouseHook] Grabbed physical mouse exclusively: {}",
            dev_path
        );

        self.running.store(true, Ordering::Release);
        let running = self.running.clone();
        let on_event = self.on_event.clone();

        let handle = thread::Builder::new()
            .name("MouseHook".to_string())
            .spawn(move || {
                use nix::poll::{poll, PollFd, PollFlags, PollTimeout};
                let mut fds = [PollFd::new(unsafe { std::os::fd::BorrowedFd::borrow_raw(fd) }, PollFlags::POLLIN)];

                let mut pressed_keys = std::collections::HashSet::new();
                let mut pending_dx = 0i32;
                let mut pending_dy = 0i32;

                let hires_scroll = dev
                    .supported_relative_axes()
                    .map(|axes| {
                        axes.contains(RelativeAxisType::REL_WHEEL_HI_RES)
                    })
                    .unwrap_or(false);
                let mut hscroll_accum = 0.0f32;

                while running.load(Ordering::Acquire) {
                    let timeout = PollTimeout::try_from(Duration::from_millis(200)).unwrap_or(PollTimeout::NONE);
                    match poll(&mut fds, timeout) {
                        Ok(n) if n > 0 => {
                            match dev.fetch_events() {
                                Ok(events) => {
                                    for event in events {
                                        let mut should_forward = true;

                                        // 1. Process virtual coordinates if it's relative motion
                                        if event.event_type() == EventType::RELATIVE {
                                            let code = RelativeAxisType(event.code());
                                            if code == RelativeAxisType::REL_X || code == RelativeAxisType::REL_Y {
                                                let dx = if code == RelativeAxisType::REL_X { event.value() } else { 0 };
                                                let dy = if code == RelativeAxisType::REL_Y { event.value() } else { 0 };

                                                // Only accumulate when NOT forwarding (else peer already got it below)
                                                if !crate::flow::FLOW_MANAGER.is_forwarding_to_remote() {
                                                    let enabled = crate::flow::FLOW_MANAGER.flow_enabled.load(Ordering::Relaxed);
                                                    if enabled {
                                                        if let Some(edge_ev) = crate::flow::FLOW_MANAGER.handle_raw_motion(dx, dy) {
                                                            let mode = crate::flow::FLOW_MANAGER.flow_mouse_mode.read().unwrap().clone();
                                                            if mode == "hardware" {
                                                                let hold_key = crate::flow::FLOW_MANAGER.flow_hold_key.read().unwrap().clone();
                                                                let ctrl_only = crate::flow::FLOW_MANAGER.flow_hold_ctrl_only.load(Ordering::Relaxed);
                                                                let mut satisfied = crate::flow::switching::is_hold_key_satisfied(&hold_key);
                                                                if ctrl_only {
                                                                    satisfied = satisfied && crate::flow::edge::is_ctrl_held();
                                                                }
                                                                if satisfied {
                                                                    crate::flow::edge::emit_edge_event(edge_ev);
                                                                }
                                                            } else {
                                                                // Legacy/existing software mode logic:
                                                                if let Some(peer) = crate::flow::topology::resolve_peer(edge_ev) {
                                                                    log::info!("[Flow] Software mode edge transition → {}", peer.peer_id);
                                                                    crate::flow::FLOW_MANAGER.set_active_peer(Some(peer.peer_id.clone()));

                                                                    // Reset coordinates to opposite edge to prevent loop bouncing
                                                                    let sw = *crate::flow::FLOW_MANAGER.screen_width.read().unwrap();
                                                                    let sh = *crate::flow::FLOW_MANAGER.screen_height.read().unwrap();
                                                                    let mut vx = crate::flow::FLOW_MANAGER.virtual_x.lock_safe();
                                                                    let mut vy = crate::flow::FLOW_MANAGER.virtual_y.lock_safe();
                                                                    match edge_ev {
                                                                        crate::flow::edge::EdgeEvent::Left => *vx = sw - 20,
                                                                        crate::flow::edge::EdgeEvent::Right => *vx = 20,
                                                                        crate::flow::edge::EdgeEvent::Top => *vy = sh - 20,
                                                                        crate::flow::edge::EdgeEvent::Bottom => *vy = 20,
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // 2. Intercept and redirect if forwarding is active
                                        let is_forwarding = crate::flow::FLOW_MANAGER.is_forwarding_to_remote();
                                        if is_forwarding {
                                            if let Some(peer_name) = crate::flow::FLOW_MANAGER.get_active_peer_name() {
                                                let flow_evt = match event.event_type() {
                                                    EventType::KEY => Some(crate::flow::network::FlowEvent::MouseButton { code: event.code(), value: event.value() }),
                                                    EventType::RELATIVE => {
                                                        let code = RelativeAxisType(event.code());
                                                        if code == RelativeAxisType::REL_X {
                                                            pending_dx += event.value();
                                                            None
                                                        } else if code == RelativeAxisType::REL_Y {
                                                            pending_dy += event.value();
                                                            None
                                                        } else if code == RelativeAxisType::REL_WHEEL {
                                                            Some(crate::flow::network::FlowEvent::MouseScroll { horizontal: false, delta: event.value() })
                                                        } else if code == RelativeAxisType::REL_HWHEEL {
                                                            Some(crate::flow::network::FlowEvent::MouseScroll { horizontal: true, delta: event.value() })
                                                        } else {
                                                            None
                                                        }
                                                    }
                                                    EventType::SYNCHRONIZATION => {
                                                        if pending_dx != 0 || pending_dy != 0 {
                                                            let evt = crate::flow::network::FlowEvent::MouseMove { dx: pending_dx, dy: pending_dy };
                                                            let _ = crate::flow::network::send_event_to_peer(&peer_name, &evt);
                                                            pending_dx = 0;
                                                            pending_dy = 0;
                                                        }
                                                        None
                                                    }
                                                    _ => None,
                                                };
                                                if let Some(evt) = flow_evt {
                                                    let _ = crate::flow::network::send_event_to_peer(&peer_name, &evt);
                                                }
                                            }
                                            should_forward = false;
                                        } else {
                                            pending_dx = 0;
                                            pending_dy = 0;
                                            match event.event_type() {
                                                EventType::KEY => {
                                                    let key = Key(event.code());
                                                    let down = event.value() == 1;

                                                    // Side buttons, middle button, etc.
                                                    if key == Key::BTN_SIDE || key == Key::BTN_EXTRA || key == Key::BTN_MIDDLE {
                                                        let is_blocked = blocked_buttons.lock_safe().contains(&key);
                                                        on_event(MouseHookEvent::Button { key, down });
                                                        if is_blocked {
                                                            should_forward = false;
                                                        }
                                                    }

                                                if should_forward {
                                                    if event.value() != 0 {
                                                        pressed_keys.insert(event.code());
                                                    } else {
                                                        pressed_keys.remove(&event.code());
                                                    }
                                                }
                                            }
                                            EventType::RELATIVE => {
                                                let code = RelativeAxisType(event.code());
                                                let mut value = event.value();

                                                if code == RelativeAxisType::REL_X || code == RelativeAxisType::REL_Y {
                                                    if gesture_active.load(Ordering::Relaxed) {
                                                        // Suppress cursor move while gesture is active, accumulate dx/dy instead
                                                        let dx = if code == RelativeAxisType::REL_X { value } else { 0 };
                                                        let dy = if code == RelativeAxisType::REL_Y { value } else { 0 };
                                                        on_event(MouseHookEvent::Relative { dx, dy });
                                                        should_forward = false;
                                                    }
                                                } else if code == RelativeAxisType::REL_WHEEL_HI_RES {
                                                    if hires_scroll {
                                                        let mut val = value;
                                                        if invert_vscroll.load(Ordering::Relaxed) {
                                                            val = -val;
                                                        }
                                                        if let Some(uinput_lock) = uinput_device.lock_safe().as_mut() {
                                                            let evs = [
                                                                InputEvent::new(EventType::RELATIVE, code.0, val),
                                                                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                                                            ];
                                                            let _ = uinput_lock.emit(&evs);
                                                        }
                                                        should_forward = false;
                                                    }
                                                } else if code == RelativeAxisType::REL_HWHEEL_HI_RES {
                                                    if hires_scroll {
                                                        let mut val = value;
                                                        if invert_hscroll.load(Ordering::Relaxed) {
                                                            val = -val;
                                                        }
                                                        if !block_hscroll.load(Ordering::Relaxed) {
                                                            if let Some(uinput_lock) = uinput_device.lock_safe().as_mut() {
                                                                let evs = [
                                                                    InputEvent::new(EventType::RELATIVE, code.0, val),
                                                                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                                                                ];
                                                                let _ = uinput_lock.emit(&evs);
                                                            }
                                                        }
                                                        hscroll_accum += val as f32 / 120.0;
                                                        let ticks = hscroll_accum.trunc() as i32;
                                                        if ticks != 0 {
                                                            hscroll_accum -= ticks as f32;
                                                            on_event(MouseHookEvent::Scroll {
                                                                horizontal: true,
                                                                delta: ticks,
                                                            });
                                                        }
                                                        should_forward = false;
                                                    }
                                                } else if code == RelativeAxisType::REL_WHEEL {
                                                    if hires_scroll {
                                                        should_forward = false;
                                                    } else {
                                                        if invert_vscroll.load(Ordering::Relaxed) {
                                                            value = -value;
                                                        }
                                                        on_event(MouseHookEvent::Scroll {
                                                            horizontal: false,
                                                            delta: value,
                                                        });
                                                        if let Some(uinput_lock) = uinput_device.lock_safe().as_mut() {
                                                            let evs = [
                                                                InputEvent::new(EventType::RELATIVE, code.0, value),
                                                                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                                                            ];
                                                            let _ = uinput_lock.emit(&evs);
                                                        }
                                                        should_forward = false;
                                                    }
                                                } else if code == RelativeAxisType::REL_HWHEEL {
                                                    if hires_scroll {
                                                        should_forward = false;
                                                    } else {
                                                        if invert_hscroll.load(Ordering::Relaxed) {
                                                            value = -value;
                                                        }
                                                        on_event(MouseHookEvent::Scroll {
                                                            horizontal: true,
                                                            delta: value,
                                                        });
                                                        if !block_hscroll.load(Ordering::Relaxed) {
                                                            if let Some(uinput_lock) = uinput_device.lock_safe().as_mut() {
                                                                let evs = [
                                                                    InputEvent::new(EventType::RELATIVE, code.0, value),
                                                                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                                                                ];
                                                                let _ = uinput_lock.emit(&evs);
                                                            }
                                                        }
                                                        should_forward = false;
                                                    }
                                                }
                                            }
                                            _ => {}
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
                                            log::info!("[MouseHook] Physical mouse on {} unplugged/disconnected.", dev_path);
                                        } else {
                                            log::warn!("[MouseHook] Read error on {}: {}. Releasing grab.", dev_path, e);
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Ok(_) => {} // Timeout reached
                        Err(e) => {
                            if e != nix::errno::Errno::EINTR {
                                log::error!("[MouseHook] Poll error on {}: {}. Releasing grab.", dev_path, e);
                                break;
                            }
                        }
                    }
                }

                // Release any stuck buttons on the virtual uinput device
                if !pressed_keys.is_empty() {
                    log::info!("[MouseHook] Releasing {} stuck buttons on uinput during shutdown", pressed_keys.len());
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
                log::info!("[MouseHook] Ungrabbed physical mouse: {}", dev_path);
                running.store(false, Ordering::Release);
            })
            .expect("Failed to spawn MouseHook thread");

        self.thread_handle = Some(handle);
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    pub fn stop(&mut self) {
        log::info!("[MouseHook] Stopping listener...");
        self.running.store(false, Ordering::Release);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
