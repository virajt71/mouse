use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomizingButton {
    Middle,
    Top,
    Forward,
    Back,
    Thumbwheel,
    Thumb,
}

impl CustomizingButton {
    pub fn config_keys(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Thumb => ("gesture", "gesture_enabled", "none"),
            Self::Forward => ("xbutton2", "xbutton2_gesture_enabled", "mouse_forward_click"),
            Self::Back => ("xbutton1", "xbutton1_gesture_enabled", "mouse_back_click"),
            Self::Top => ("mode_shift", "top_gesture_enabled", "switch_scroll_mode"),
            Self::Middle => ("middle", "middle_gesture_enabled", "mouse_middle_click"),
            _ => ("", "", ""),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    MiddleClick,
    ModeShift,
    Forward,
    Back,
    HorizontalScroll,
    Gestures,
    Keystroke,
    Disabled,
}

impl ButtonAction {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::MiddleClick => "Middle click",
            Self::ModeShift => "Shift wheel mode",
            Self::Forward => "Forward click",
            Self::Back => "Back click",
            Self::HorizontalScroll => "Horizontal scroll",
            Self::Gestures => "Gestures",
            Self::Keystroke => "Keyboard shortcut",
            Self::Disabled => "Disabled",
        }
    }
}

pub fn get_button_keys(btn: CustomizingButton) -> (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str) {
    match btn {
        CustomizingButton::Middle => (
            "middle",
            "middle_gesture_enabled",
            "middle_gesture_up",
            "middle_gesture_down",
            "middle_gesture_left",
            "middle_gesture_right",
        ),
        CustomizingButton::Top => (
            "mode_shift",
            "top_gesture_enabled",
            "top_gesture_up",
            "top_gesture_down",
            "top_gesture_left",
            "top_gesture_right",
        ),
        CustomizingButton::Forward => (
            "xbutton2",
            "xbutton2_gesture_enabled",
            "xbutton2_gesture_up",
            "xbutton2_gesture_down",
            "xbutton2_gesture_left",
            "xbutton2_gesture_right",
        ),
        CustomizingButton::Back => (
            "xbutton1",
            "xbutton1_gesture_enabled",
            "xbutton1_gesture_up",
            "xbutton1_gesture_down",
            "xbutton1_gesture_left",
            "xbutton1_gesture_right",
        ),
        CustomizingButton::Thumb => (
            "gesture",
            "gesture_enabled",
            "gesture_up",
            "gesture_down",
            "gesture_left",
            "gesture_right",
        ),
        CustomizingButton::Thumbwheel => (
            "hscroll",
            "hscroll_gesture_enabled",
            "hscroll_gesture_up",
            "hscroll_gesture_down",
            "hscroll_gesture_left",
            "hscroll_gesture_right",
        ),
    }
}

pub fn mapping_to_action(btn: CustomizingButton, mappings: &std::collections::HashMap<String, String>) -> ButtonAction {
    let (base_key, gesture_enabled_key, _, _, _, _) = get_button_keys(btn);
    let val = mappings.get(base_key).map(|s| s.as_str()).unwrap_or("none");
    let gesture_enabled = mappings.get(gesture_enabled_key).map(|s| s == "true").unwrap_or(false);

    // Check gesture mode via explicit flag, legacy placeholder, or any configured direction.
    let is_gesture = gesture_enabled
        || val == "gestures"
        || {
            let (_, _, up_k, down_k, left_k, right_k) = get_button_keys(btn);
            [up_k, down_k, left_k, right_k].iter().any(|k| {
                mappings.get(*k).map(|v| v.as_str() != "none").unwrap_or(false)
            })
        };

    if is_gesture {
        ButtonAction::Gestures
    } else if val == "none" {
        ButtonAction::Disabled
    } else if val.starts_with("custom:") {
        ButtonAction::Keystroke
    } else {
        match val {
            "mouse_middle_click" => ButtonAction::MiddleClick,
            "switch_scroll_mode" => ButtonAction::ModeShift,
            "mouse_forward_click" => ButtonAction::Forward,
            "mouse_back_click" => ButtonAction::Back,
            "hscroll" => ButtonAction::HorizontalScroll,
            _ => ButtonAction::Disabled,
        }
    }
}

pub fn egui_key_to_string(key: egui::Key) -> String {
    match key {
        egui::Key::A => "a".to_string(),
        egui::Key::B => "b".to_string(),
        egui::Key::C => "c".to_string(),
        egui::Key::D => "d".to_string(),
        egui::Key::E => "e".to_string(),
        egui::Key::F => "f".to_string(),
        egui::Key::G => "g".to_string(),
        egui::Key::H => "h".to_string(),
        egui::Key::I => "i".to_string(),
        egui::Key::J => "j".to_string(),
        egui::Key::K => "k".to_string(),
        egui::Key::L => "l".to_string(),
        egui::Key::M => "m".to_string(),
        egui::Key::N => "n".to_string(),
        egui::Key::O => "o".to_string(),
        egui::Key::P => "p".to_string(),
        egui::Key::Q => "q".to_string(),
        egui::Key::R => "r".to_string(),
        egui::Key::S => "s".to_string(),
        egui::Key::T => "t".to_string(),
        egui::Key::U => "u".to_string(),
        egui::Key::V => "v".to_string(),
        egui::Key::W => "w".to_string(),
        egui::Key::X => "x".to_string(),
        egui::Key::Y => "y".to_string(),
        egui::Key::Z => "z".to_string(),
        egui::Key::Num0 => "0".to_string(),
        egui::Key::Num1 => "1".to_string(),
        egui::Key::Num2 => "2".to_string(),
        egui::Key::Num3 => "3".to_string(),
        egui::Key::Num4 => "4".to_string(),
        egui::Key::Num5 => "5".to_string(),
        egui::Key::Num6 => "6".to_string(),
        egui::Key::Num7 => "7".to_string(),
        egui::Key::Num8 => "8".to_string(),
        egui::Key::Num9 => "9".to_string(),
        egui::Key::F1 => "f1".to_string(),
        egui::Key::F2 => "f2".to_string(),
        egui::Key::F3 => "f3".to_string(),
        egui::Key::F4 => "f4".to_string(),
        egui::Key::F5 => "f5".to_string(),
        egui::Key::F6 => "f6".to_string(),
        egui::Key::F7 => "f7".to_string(),
        egui::Key::F8 => "f8".to_string(),
        egui::Key::F9 => "f9".to_string(),
        egui::Key::F10 => "f10".to_string(),
        egui::Key::F11 => "f11".to_string(),
        egui::Key::F12 => "f12".to_string(),
        egui::Key::ArrowLeft => "left".to_string(),
        egui::Key::ArrowRight => "right".to_string(),
        egui::Key::ArrowUp => "up".to_string(),
        egui::Key::ArrowDown => "down".to_string(),
        egui::Key::Home => "home".to_string(),
        egui::Key::End => "end".to_string(),
        egui::Key::PageUp => "pageup".to_string(),
        egui::Key::PageDown => "pagedown".to_string(),
        egui::Key::Backspace => "backspace".to_string(),
        egui::Key::Delete => "delete".to_string(),
        egui::Key::Tab => "tab".to_string(),
        egui::Key::Space => "space".to_string(),
        egui::Key::Enter => "enter".to_string(),
        egui::Key::Escape => "escape".to_string(),
        _ => "".to_string(),
    }
}

pub fn is_valid_combo(combo: &str) -> bool {
    if combo.is_empty() {
        return false;
    }
    let parts: Vec<String> = combo.split('+').map(|s| s.trim().to_lowercase()).collect();
    for part in parts {
        if part != "ctrl" && part != "shift" && part != "alt" && part != "meta" && !part.is_empty() {
            return true;
        }
    }
    false
}
