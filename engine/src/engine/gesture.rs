use crate::lock_ext::MutexExt;
use evdev::Key;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use super::Engine;
use crate::hidpp::HidppEvent;
use crate::input::{get_mouse_button_key, is_mouse_button_action, MouseHookEvent};

impl Engine {
    pub fn handle_mouse_hook_event(&self, event: MouseHookEvent) {
        match event {
            MouseHookEvent::Button { key, down } => {
                let btn_key = if key == Key::BTN_MIDDLE {
                    "middle"
                } else if key == Key::BTN_SIDE {
                    "xbutton1"
                } else if key == Key::BTN_EXTRA {
                    "xbutton2"
                } else {
                    return;
                };

                let gestures_enabled = {
                    let active = self.inner.active_mappings.read().unwrap();
                    let enabled_key = if btn_key == "middle" {
                        "middle_gesture_enabled"
                    } else if btn_key == "xbutton1" {
                        "xbutton1_gesture_enabled"
                    } else {
                        "xbutton2_gesture_enabled"
                    };

                    let explicit = active.get(enabled_key).map(|s| s.as_ref());
                    match explicit {
                        Some("false") => {
                            let prefix = if btn_key == "middle" {
                                "middle_gesture_"
                            } else if btn_key == "xbutton1" {
                                "xbutton1_gesture_"
                            } else {
                                "xbutton2_gesture_"
                            };
                            ["left", "right", "up", "down"].iter().any(|dir| {
                                let k = format!("{}{}", prefix, dir);
                                active
                                    .get(&k)
                                    .map(|v| v.as_ref() != "none")
                                    .unwrap_or(false)
                            })
                        }
                        Some("true") => true,
                        _ => true,
                    }
                };

                if gestures_enabled {
                    if down {
                        self.handle_gesture_down();
                        self.inner.gesture_state.lock_safe().button = Some(btn_key.to_string());
                    } else {
                        let is_current = {
                            let state = self.inner.gesture_state.lock_safe();
                            state.button.as_ref() == Some(&btn_key.to_string())
                        };
                        if is_current {
                            self.handle_gesture_up();
                        }
                    }
                    return;
                }

                let mapping = {
                    let active = self.inner.active_mappings.read().unwrap();
                    active.get(btn_key).cloned()
                };

                if let Some(action_id) = mapping {
                    if action_id.as_ref() == "none" {
                        return;
                    }
                    if is_mouse_button_action(action_id.as_ref()) {
                        if let Some(sim_key) = get_mouse_button_key(action_id.as_ref()) {
                            if down {
                                self.inner.key_simulator.inject_mouse_down(sim_key);
                            } else {
                                self.inner.key_simulator.inject_mouse_up(sim_key);
                            }
                        }
                    } else if down {
                        self.execute_engine_action(action_id.as_ref());
                    }
                }
            }
            MouseHookEvent::Scroll { horizontal, delta } => {
                if horizontal {
                    let action_key = if delta < 0 {
                        "hscroll_right"
                    } else {
                        "hscroll_left"
                    };

                    let mapping = {
                        let active = self.inner.active_mappings.read().unwrap();
                        active.get(action_key).cloned()
                    };

                    if let Some(action_id) = mapping {
                        if action_id.as_ref() != "none" {
                            self.handle_hscroll_event(delta, action_id.as_ref());
                        }
                    }
                }
            }
            MouseHookEvent::Relative { dx, dy } => {
                if self.inner.gesture_active_arc.load(Ordering::Relaxed) {
                    self.handle_gesture_move(dx as i16, dy as i16, "evdev");
                }
            }
        }
    }

    pub fn handle_hid_event(&self, event: HidppEvent) {
        match event {
            HidppEvent::GestureDown => {
                let (enabled, mapping) = {
                    let active = self.inner.active_mappings.read().unwrap();
                    let enabled = match active.get("gesture_enabled").map(|s| s.as_ref()) {
                        Some("false") => [
                            "gesture_left",
                            "gesture_right",
                            "gesture_up",
                            "gesture_down",
                        ]
                        .iter()
                        .any(|k| {
                            active
                                .get(*k)
                                .map(|v| v.as_ref() != "none")
                                .unwrap_or(false)
                        }),
                        _ => true,
                    };
                    let mapping = active.get("gesture").cloned();
                    (enabled, mapping)
                };
                if enabled {
                    self.handle_gesture_down();
                } else if let Some(ref action_id) = mapping {
                    if action_id.as_ref() != "none" {
                        if is_mouse_button_action(action_id.as_ref()) {
                            if let Some(sim_key) = get_mouse_button_key(action_id.as_ref()) {
                                log::info!(
                                    "[Engine] Pressing mouse button for gesture action: {:?}",
                                    sim_key
                                );
                                self.inner.key_simulator.inject_mouse_down(sim_key);
                            }
                        } else {
                            log::info!(
                                "[Engine] Executing gesture simple click action: {}",
                                action_id
                            );
                            self.execute_engine_action(action_id.as_ref());
                        }
                    }
                }
            }
            HidppEvent::GestureUp => {
                let (enabled, mapping) = {
                    let active = self.inner.active_mappings.read().unwrap();
                    let enabled = match active.get("gesture_enabled").map(|s| s.as_ref()) {
                        Some("false") => [
                            "gesture_left",
                            "gesture_right",
                            "gesture_up",
                            "gesture_down",
                        ]
                        .iter()
                        .any(|k| {
                            active
                                .get(*k)
                                .map(|v| v.as_ref() != "none")
                                .unwrap_or(false)
                        }),
                        _ => true,
                    };
                    let mapping = active.get("gesture").cloned();
                    (enabled, mapping)
                };
                if enabled {
                    self.handle_gesture_up();
                } else if let Some(ref action_id) = mapping {
                    if action_id.as_ref() != "none" && is_mouse_button_action(action_id.as_ref()) {
                        if let Some(sim_key) = get_mouse_button_key(action_id.as_ref()) {
                            log::info!(
                                "[Engine] Releasing mouse button for gesture action: {:?}",
                                sim_key
                            );
                            self.inner.key_simulator.inject_mouse_up(sim_key);
                        }
                    }
                }
            }
            HidppEvent::GestureMove { dx, dy } => {
                let enabled = {
                    let btn_key = self.inner.gesture_state.lock_safe().button.clone();
                    let active = self.inner.active_mappings.read().unwrap();
                    let enabled_key = match btn_key.as_deref() {
                        Some("middle") => "middle_gesture_enabled",
                        Some("xbutton1") => "xbutton1_gesture_enabled",
                        Some("xbutton2") => "xbutton2_gesture_enabled",
                        _ => "gesture_enabled",
                    };
                    match active.get(enabled_key).map(|s| s.as_ref()) {
                        Some("false") => {
                            let prefix = match btn_key.as_deref() {
                                Some("middle") => "middle_gesture_",
                                Some("xbutton1") => "xbutton1_gesture_",
                                Some("xbutton2") => "xbutton2_gesture_",
                                _ => "gesture_",
                            };
                            ["left", "right", "up", "down"].iter().any(|dir| {
                                let k = format!("{}{}", prefix, dir);
                                active
                                    .get(&k)
                                    .map(|v| v.as_ref() != "none")
                                    .unwrap_or(false)
                            })
                        }
                        _ => true,
                    }
                };
                if enabled {
                    self.handle_gesture_move(dx, dy, "hid_rawxy");
                } else {
                    self.inner
                        .key_simulator
                        .inject_relative_move(dx as i32, dy as i32);
                }
            }
            HidppEvent::ModeShiftDown => {
                log::debug!("[Engine] HID ModeShift button down");
                let mapping = self
                    .inner
                    .active_mappings
                    .read()
                    .unwrap()
                    .get("mode_shift")
                    .cloned();
                if let Some(ref action_id) = mapping {
                    self.execute_engine_action(action_id.as_ref());
                }
            }
            HidppEvent::ModeShiftUp => {
                log::debug!("[Engine] HID ModeShift button up");
            }
            HidppEvent::BacklightChanged { enabled, effect_id } => {
                self.apply_backlight_from_hid(enabled, effect_id);
            }
            HidppEvent::HostChannelChanged(ch) => {
                log::info!("[Engine] Host channel changed to {}", ch);
                let cfg_opt = {
                    let mut cfg = self.inner.config.lock_safe();
                    if cfg.settings.flow_local_channel_index != ch {
                        cfg.settings.flow_local_channel_index = ch;
                        let _ = cfg.save();
                        Some(cfg.clone())
                    } else {
                        None
                    }
                };
                if let Some(cfg) = cfg_opt {
                    self.increment_config_generation(&cfg);
                }
                self.update_cached_device_state();
            }
            HidppEvent::BatteryChanged(b) => {
                log::info!("[HID++] Battery reading: {}% ({})", b.percentage, b.status);
            }
        }
    }

    pub fn handle_gesture_down(&self) {
        log::debug!("[Engine] Gesture track: button down");
        self.inner.gesture_active_arc.store(true, Ordering::Relaxed);
        self.inner.gesture_tracking.store(true, Ordering::Relaxed);
        self.inner.gesture_triggered.store(false, Ordering::Relaxed);
        {
            let mut state = self.inner.gesture_state.lock_safe();
            state.delta_x = 0.0;
            state.delta_y = 0.0;
            state.last_move_at = Instant::now();
            state.input_source = None;
        }
    }

    pub fn handle_gesture_up(&self) {
        log::debug!("[Engine] Gesture track: button up");

        // Always clear button and input_source — even if gesture_active was already false.
        // Without this, a stale `button` leaks into the next gesture cycle (§3.2).
        let triggered;
        let btn_key;
        {
            let mut state = self.inner.gesture_state.lock_safe();
            btn_key = state.button.take().unwrap_or_else(|| "gesture".to_string());
            state.input_source = None;
            triggered = self.inner.gesture_triggered.load(Ordering::Relaxed);
        }

        if !self.inner.gesture_active_arc.swap(false, Ordering::Relaxed) {
            // Was already inactive — button has been cleared above, nothing else to do.
            return;
        }

        self.inner.gesture_tracking.store(false, Ordering::Relaxed);

        if !triggered {
            let mapping = self
                .inner
                .active_mappings
                .read()
                .unwrap()
                .get(&btn_key)
                .cloned();
            if let Some(ref action_id) = mapping {
                log::info!(
                    "[Engine] Executing {} click fallback action: {}",
                    btn_key,
                    action_id
                );
                self.execute_engine_action(action_id.as_ref());
            }
        }
    }

    pub fn handle_gesture_move(&self, dx: i16, dy: i16, source: &str) {
        let threshold = self.inner.cached_gesture_threshold.load(Ordering::Relaxed) as f32;
        let deadzone = self.inner.cached_gesture_deadzone.load(Ordering::Relaxed) as f32;
        let timeout =
            Duration::from_millis(self.inner.cached_gesture_timeout_ms.load(Ordering::Relaxed));
        let cooldown = Duration::from_millis(
            self.inner
                .cached_gesture_cooldown_ms
                .load(Ordering::Relaxed),
        );

        let now = Instant::now();

        let mut state = self.inner.gesture_state.lock_safe();

        if now < state.cooldown_until {
            return;
        }

        let idle_time = now.duration_since(state.last_move_at);
        if idle_time > timeout {
            log::debug!("[Engine] Segment timeout, resetting accumulator");
            self.inner.gesture_tracking.store(true, Ordering::Relaxed);
            self.inner.gesture_triggered.store(false, Ordering::Relaxed);
            state.delta_x = 0.0;
            state.delta_y = 0.0;
        }
        state.last_move_at = now;

        if let Some(ref active_source) = state.input_source {
            if active_source == "evdev" && source == "hid_rawxy" {
                log::debug!("[Engine] Promoting source to hid_rawxy");
                state.input_source = Some(source.to_string());
                state.delta_x = 0.0;
                state.delta_y = 0.0;
            } else if active_source != source {
                return;
            }
        } else {
            state.input_source = Some(source.to_string());
        }

        if !self.inner.gesture_tracking.load(Ordering::Relaxed) {
            self.inner.gesture_tracking.store(true, Ordering::Relaxed);
            self.inner.gesture_triggered.store(false, Ordering::Relaxed);
            state.delta_x = 0.0;
            state.delta_y = 0.0;
        }

        state.delta_x += dx as f32;
        state.delta_y += dy as f32;

        let accum_x = state.delta_x;
        let accum_y = state.delta_y;

        let abs_x = accum_x.abs();
        let abs_y = accum_y.abs();
        let dominant = abs_x.max(abs_y);

        if dominant < threshold {
            return;
        }

        if self.inner.gesture_triggered.load(Ordering::Relaxed) {
            return;
        }

        let cross_limit = deadzone.max(dominant * 0.35);

        let gesture_action = if abs_x > abs_y {
            if abs_y > cross_limit {
                None
            } else if accum_x > 0.0 {
                Some("gesture_right")
            } else {
                Some("gesture_left")
            }
        } else {
            if abs_x > cross_limit {
                None
            } else if accum_y > 0.0 {
                Some("gesture_down")
            } else {
                Some("gesture_up")
            }
        };

        if let Some(action_key) = gesture_action {
            self.inner.gesture_triggered.store(true, Ordering::Relaxed);
            state.cooldown_until = now + cooldown;

            let btn_key = state
                .button
                .clone()
                .unwrap_or_else(|| "gesture".to_string());

            drop(state);

            let resolved_action_key = if btn_key == "gesture" {
                action_key.to_string()
            } else {
                format!("{}_{}", btn_key, action_key)
            };

            log::info!(
                "[Engine] Gesture detected for {}: {}",
                btn_key,
                resolved_action_key
            );

            let mapping = self
                .inner
                .active_mappings
                .read()
                .unwrap()
                .get(&resolved_action_key)
                .cloned();
            if let Some(ref action_id) = mapping {
                self.execute_engine_action(action_id.as_ref());
            }
        }
    }
}
