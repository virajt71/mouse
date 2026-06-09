use super::button_keys::{CustomizingButton, get_button_keys};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniversalButtonOption {
    Gesture,
    TaskView,
    ShowHideDesktop,
    ScreenCapture,
    PrintScreen,
    SwitchApplication,
    KeyboardShortcut,
    ActionCenter,
    AdvancedClick,
    Back,
    BrightnessDown,
    BrightnessUp,
    Calculator,
    ChangePointerSpeed,
    CloseWindow,
    Copy,
    Cut,
    DesktopLeft,
    DesktopRight,
    Dictation,
    DoNothing,
    Emoji,
    EmojisMenu,
    Forward,
    InputLanguage,
    Lock,
    MaximizeWindow,
    MiddleButton,
    MinimizeWindow,
    MuteUnmuteSpeaker,
    NewBrowserTab,
    Next,
    OpenApplication,
    OpenFile,
    OpenFolder,
    Paste,
    PlayPause,
    Previous,
    Redo,
    RightCtrl,
    ScreenSnip,
    ShiftWheelMode,
    ThisPC,
    Undo,
    VolumeDown,
    VolumeUp,
    ZoomIn,
    ZoomOut,
}

impl UniversalButtonOption {
    pub fn display_name(self, btn: CustomizingButton) -> &'static str {
        match self {
            Self::Gesture => {
                if btn == CustomizingButton::Thumb || btn == CustomizingButton::Top || btn == CustomizingButton::Middle {
                    "Gestures"
                } else {
                    "Gesture"
                }
            }
            Self::TaskView => "Task view",
            Self::ShowHideDesktop => "Show/hide desktop",
            Self::ScreenCapture => "Screen capture",
            Self::PrintScreen => "Print screen",
            Self::SwitchApplication => "Switch application",
            Self::KeyboardShortcut => "Keyboard shortcut",
            Self::ActionCenter => "Action center",
            Self::AdvancedClick => "Advanced click",
            Self::Back => "Back",
            Self::BrightnessDown => "Brightness down",
            Self::BrightnessUp => "Brightness up",
            Self::Calculator => "Calculator",
            Self::ChangePointerSpeed => "Change pointer speed",
            Self::CloseWindow => "Close window",
            Self::Copy => "Copy",
            Self::Cut => "Cut",
            Self::DesktopLeft => "Desktop left",
            Self::DesktopRight => "Desktop right",
            Self::Dictation => "Dictation",
            Self::DoNothing => "Do nothing",
            Self::Emoji => "Emoji",
            Self::EmojisMenu => "Emoji's menu",
            Self::Forward => "Forward",
            Self::InputLanguage => "Input Language",
            Self::Lock => "Lock",
            Self::MaximizeWindow => "Maximize window",
            Self::MiddleButton => "Middle button",
            Self::MinimizeWindow => "Minimize window",
            Self::MuteUnmuteSpeaker => "Mute/Unmute speaker",
            Self::NewBrowserTab => "New browser tab",
            Self::Next => "Next",
            Self::OpenApplication => "Open application",
            Self::OpenFile => "Open file",
            Self::OpenFolder => "Open folder",
            Self::Paste => "Paste",
            Self::PlayPause => "Play/Pause",
            Self::Previous => "Previous",
            Self::Redo => "Redo",
            Self::RightCtrl => "Right Ctrl",
            Self::ScreenSnip => "Screen snip",
            Self::ShiftWheelMode => "Shift wheel mode",
            Self::ThisPC => "This PC",
            Self::Undo => "Undo",
            Self::VolumeDown => "Volume down",
            Self::VolumeUp => "Volume up",
            Self::ZoomIn => "Zoom in",
            Self::ZoomOut => "Zoom out",
        }
    }
}

impl CustomizingButton {
    pub fn recommended_options(self) -> &'static [UniversalButtonOption] {
        match self {
            Self::Thumb => &[
                UniversalButtonOption::Gesture,
                UniversalButtonOption::TaskView,
                UniversalButtonOption::ShowHideDesktop,
                UniversalButtonOption::ScreenCapture,
                UniversalButtonOption::PrintScreen,
                UniversalButtonOption::SwitchApplication,
                UniversalButtonOption::KeyboardShortcut,
            ],
            Self::Forward => &[
                UniversalButtonOption::Forward,
                UniversalButtonOption::Paste,
                UniversalButtonOption::VolumeUp,
                UniversalButtonOption::Redo,
                UniversalButtonOption::KeyboardShortcut,
            ],
            Self::Back => &[
                UniversalButtonOption::Back,
                UniversalButtonOption::Copy,
                UniversalButtonOption::VolumeDown,
                UniversalButtonOption::Undo,
                UniversalButtonOption::KeyboardShortcut,
            ],
            Self::Top => &[
                UniversalButtonOption::ShiftWheelMode,
                UniversalButtonOption::TaskView,
                UniversalButtonOption::Gesture,
                UniversalButtonOption::ScreenCapture,
                UniversalButtonOption::PrintScreen,
                UniversalButtonOption::KeyboardShortcut,
            ],
            Self::Middle => &[
                UniversalButtonOption::MiddleButton,
                UniversalButtonOption::ShiftWheelMode,
                UniversalButtonOption::TaskView,
                UniversalButtonOption::ShowHideDesktop,
                UniversalButtonOption::Gesture,
                UniversalButtonOption::KeyboardShortcut,
            ],
            _ => &[],
        }
    }

    pub fn other_options(self) -> Vec<UniversalButtonOption> {
        let recommended = self.recommended_options();
        let all_options = &[
            UniversalButtonOption::Gesture,
            UniversalButtonOption::ActionCenter,
            UniversalButtonOption::AdvancedClick,
            UniversalButtonOption::Back,
            UniversalButtonOption::BrightnessDown,
            UniversalButtonOption::BrightnessUp,
            UniversalButtonOption::Calculator,
            UniversalButtonOption::ChangePointerSpeed,
            UniversalButtonOption::CloseWindow,
            UniversalButtonOption::Copy,
            UniversalButtonOption::Cut,
            UniversalButtonOption::DesktopLeft,
            UniversalButtonOption::DesktopRight,
            UniversalButtonOption::Dictation,
            UniversalButtonOption::DoNothing,
            UniversalButtonOption::Emoji,
            UniversalButtonOption::EmojisMenu,
            UniversalButtonOption::Forward,
            UniversalButtonOption::InputLanguage,
            UniversalButtonOption::Lock,
            UniversalButtonOption::MaximizeWindow,
            UniversalButtonOption::MiddleButton,
            UniversalButtonOption::MinimizeWindow,
            UniversalButtonOption::MuteUnmuteSpeaker,
            UniversalButtonOption::NewBrowserTab,
            UniversalButtonOption::Next,
            UniversalButtonOption::OpenApplication,
            UniversalButtonOption::OpenFile,
            UniversalButtonOption::OpenFolder,
            UniversalButtonOption::Paste,
            UniversalButtonOption::PlayPause,
            UniversalButtonOption::Previous,
            UniversalButtonOption::PrintScreen,
            UniversalButtonOption::Redo,
            UniversalButtonOption::RightCtrl,
            UniversalButtonOption::ScreenCapture,
            UniversalButtonOption::ScreenSnip,
            UniversalButtonOption::ShiftWheelMode,
            UniversalButtonOption::ShowHideDesktop,
            UniversalButtonOption::SwitchApplication,
            UniversalButtonOption::TaskView,
            UniversalButtonOption::ThisPC,
            UniversalButtonOption::Undo,
            UniversalButtonOption::VolumeDown,
            UniversalButtonOption::VolumeUp,
            UniversalButtonOption::ZoomIn,
            UniversalButtonOption::ZoomOut,
        ];

        all_options.iter()
            .copied()
            .filter(|opt| !recommended.contains(opt))
            .collect()
    }
}

pub fn get_button_option(
    btn: CustomizingButton,
    mappings: &std::collections::HashMap<String, String>,
) -> UniversalButtonOption {
    let (base_key, gesture_enabled_key, _) = btn.config_keys();

    // Primary check: explicit gesture_enabled flag.
    let gesture_enabled = mappings.get(gesture_enabled_key).map(|s| s == "true").unwrap_or(false);
    if gesture_enabled {
        return UniversalButtonOption::Gesture;
    }

    // Legacy migration: if the base key is "gestures" (old placeholder), treat as gesture mode.
    if mappings.get(base_key).map(|s| s == "gestures").unwrap_or(false) {
        return UniversalButtonOption::Gesture;
    }

    // Auto-detect: if any gesture direction is non-"none", the button is in gesture mode
    // even if gesture_enabled was never explicitly saved as "true" (e.g. old configs).
    {
        let (_, _, up_k, down_k, left_k, right_k) = get_button_keys(btn);
        let has_gesture = [up_k, down_k, left_k, right_k].iter().any(|k| {
            mappings.get(*k).map(|v| v != "none").unwrap_or(false)
        });
        if has_gesture {
            return UniversalButtonOption::Gesture;
        }
    }

    if let Some(val) = mappings.get(base_key) {
        match val.as_str() {
            "task_view" => return UniversalButtonOption::TaskView,
            "win_d" => return UniversalButtonOption::ShowHideDesktop,
            "screen_capture" => return UniversalButtonOption::ScreenCapture,
            "print_screen" => return UniversalButtonOption::PrintScreen,
            "alt_tab" => return UniversalButtonOption::SwitchApplication,
            "action_center" => return UniversalButtonOption::ActionCenter,
            "advanced_click" => return UniversalButtonOption::AdvancedClick,
            "browser_back" | "mouse_back_click" => return UniversalButtonOption::Back,
            "brightness_down" => return UniversalButtonOption::BrightnessDown,
            "brightness_up" => return UniversalButtonOption::BrightnessUp,
            "calculator" => return UniversalButtonOption::Calculator,
            "cycle_dpi" => return UniversalButtonOption::ChangePointerSpeed,
            "close_window" => return UniversalButtonOption::CloseWindow,
            "copy" => return UniversalButtonOption::Copy,
            "cut" => return UniversalButtonOption::Cut,
            "space_left" => return UniversalButtonOption::DesktopLeft,
            "space_right" => return UniversalButtonOption::DesktopRight,
            "dictation" => return UniversalButtonOption::Dictation,
            "none" => return UniversalButtonOption::DoNothing,
            "emoji" => return UniversalButtonOption::Emoji,
            "emojis_menu" => return UniversalButtonOption::EmojisMenu,
            "browser_forward" | "mouse_forward_click" => return UniversalButtonOption::Forward,
            "input_language" => return UniversalButtonOption::InputLanguage,
            "lock" => return UniversalButtonOption::Lock,
            "maximize_window" => return UniversalButtonOption::MaximizeWindow,
            "mouse_middle_click" => return UniversalButtonOption::MiddleButton,
            "minimize_window" => return UniversalButtonOption::MinimizeWindow,
            "volume_mute" => return UniversalButtonOption::MuteUnmuteSpeaker,
            "new_tab" => return UniversalButtonOption::NewBrowserTab,
            "next_track" => return UniversalButtonOption::Next,
            "open_application" => return UniversalButtonOption::OpenApplication,
            "open_file" => return UniversalButtonOption::OpenFile,
            "open_folder" => return UniversalButtonOption::OpenFolder,
            "paste" => return UniversalButtonOption::Paste,
            "play_pause" => return UniversalButtonOption::PlayPause,
            "prev_track" => return UniversalButtonOption::Previous,
            "redo" => return UniversalButtonOption::Redo,
            "right_ctrl" => return UniversalButtonOption::RightCtrl,
            "screen_snip" => return UniversalButtonOption::ScreenSnip,
            "switch_scroll_mode" => return UniversalButtonOption::ShiftWheelMode,
            "this_pc" => return UniversalButtonOption::ThisPC,
            "undo" => return UniversalButtonOption::Undo,
            "volume_down" => return UniversalButtonOption::VolumeDown,
            "volume_up" => return UniversalButtonOption::VolumeUp,
            "zoom_in" => return UniversalButtonOption::ZoomIn,
            "zoom_out" => return UniversalButtonOption::ZoomOut,
            s if s.starts_with("custom:") => return UniversalButtonOption::KeyboardShortcut,
            _ => {}
        }
    }

    match btn {
        CustomizingButton::Thumb => UniversalButtonOption::DoNothing,
        CustomizingButton::Forward => UniversalButtonOption::Forward,
        CustomizingButton::Back => UniversalButtonOption::Back,
        CustomizingButton::Top => UniversalButtonOption::ShiftWheelMode,
        CustomizingButton::Middle => UniversalButtonOption::MiddleButton,
        _ => UniversalButtonOption::DoNothing,
    }
}

pub fn save_button_option(
    btn: CustomizingButton,
    opt: UniversalButtonOption,
    mappings: &mut std::collections::HashMap<String, String>,
) {
    let (base_key, gesture_enabled_key, _) = btn.config_keys();
    if opt == UniversalButtonOption::Gesture {
        mappings.insert(gesture_enabled_key.to_string(), "true".to_string());
        // Preserve the existing click-fallback action (the base key doubles as the
        // "CLICK" slot inside gesture config). Only reset if it holds the legacy
        // "gestures" placeholder or is completely absent — never overwrite a valid action.
        let existing = mappings.get(base_key).map(|s| s.as_str()).unwrap_or("none");
        if existing == "gestures" || existing.is_empty() {
            mappings.insert(base_key.to_string(), "none".to_string());
        }
        return;
    }

    mappings.insert(gesture_enabled_key.to_string(), "false".to_string());

    // Clear gesture direction slots so auto-detect won't re-enable gesture mode.
    let (_, _, up_k, down_k, left_k, right_k) = get_button_keys(btn);
    for dir_key in [up_k, down_k, left_k, right_k] {
        mappings.insert(dir_key.to_string(), "none".to_string());
    }

    let val = match opt {
        UniversalButtonOption::Gesture => "gestures",
        UniversalButtonOption::TaskView => "task_view",
        UniversalButtonOption::ShowHideDesktop => "win_d",
        UniversalButtonOption::ScreenCapture => "screen_capture",
        UniversalButtonOption::PrintScreen => "print_screen",
        UniversalButtonOption::SwitchApplication => "alt_tab",
        UniversalButtonOption::KeyboardShortcut => {
            if !mappings.get(base_key).map(|s| s.starts_with("custom:")).unwrap_or(false) {
                mappings.insert(base_key.to_string(), "none".to_string());
            }
            return;
        }
        UniversalButtonOption::ActionCenter => "action_center",
        UniversalButtonOption::AdvancedClick => "advanced_click",
        UniversalButtonOption::Back => {
            if btn == CustomizingButton::Back { "mouse_back_click" } else { "browser_back" }
        }
        UniversalButtonOption::BrightnessDown => "brightness_down",
        UniversalButtonOption::BrightnessUp => "brightness_up",
        UniversalButtonOption::Calculator => "calculator",
        UniversalButtonOption::ChangePointerSpeed => "cycle_dpi",
        UniversalButtonOption::CloseWindow => "close_window",
        UniversalButtonOption::Copy => "copy",
        UniversalButtonOption::Cut => "cut",
        UniversalButtonOption::DesktopLeft => "space_left",
        UniversalButtonOption::DesktopRight => "space_right",
        UniversalButtonOption::Dictation => "dictation",
        UniversalButtonOption::DoNothing => "none",
        UniversalButtonOption::Emoji => "emoji",
        UniversalButtonOption::EmojisMenu => "emojis_menu",
        UniversalButtonOption::Forward => {
            if btn == CustomizingButton::Forward { "mouse_forward_click" } else { "browser_forward" }
        }
        UniversalButtonOption::InputLanguage => "input_language",
        UniversalButtonOption::Lock => "lock",
        UniversalButtonOption::MaximizeWindow => "maximize_window",
        UniversalButtonOption::MiddleButton => "mouse_middle_click",
        UniversalButtonOption::MinimizeWindow => "minimize_window",
        UniversalButtonOption::MuteUnmuteSpeaker => "volume_mute",
        UniversalButtonOption::NewBrowserTab => "new_tab",
        UniversalButtonOption::Next => "next_track",
        UniversalButtonOption::OpenApplication => "open_application",
        UniversalButtonOption::OpenFile => "open_file",
        UniversalButtonOption::OpenFolder => "open_folder",
        UniversalButtonOption::Paste => "paste",
        UniversalButtonOption::PlayPause => "play_pause",
        UniversalButtonOption::Previous => "prev_track",
        UniversalButtonOption::Redo => "redo",
        UniversalButtonOption::RightCtrl => "right_ctrl",
        UniversalButtonOption::ScreenSnip => "screen_snip",
        UniversalButtonOption::ShiftWheelMode => "switch_scroll_mode",
        UniversalButtonOption::ThisPC => "this_pc",
        UniversalButtonOption::Undo => "undo",
        UniversalButtonOption::VolumeDown => "volume_down",
        UniversalButtonOption::VolumeUp => "volume_up",
        UniversalButtonOption::ZoomIn => "zoom_in",
        UniversalButtonOption::ZoomOut => "zoom_out",
    };

    mappings.insert(base_key.to_string(), val.to_string());
}
