#[derive(Debug, Clone, Copy)]
pub struct GesturePreset {
    pub name: &'static str,
    pub left: &'static str,
    pub right: &'static str,
    pub up: &'static str,
    pub down: &'static str,
    pub click: &'static str,
}

pub const GESTURE_PRESETS: &[GesturePreset] = &[
    GesturePreset {
        name: "Custom",
        left: "none",
        right: "none",
        up: "none",
        down: "none",
        click: "none",
    },
    GesturePreset {
        name: "Arrange windows",
        left: "snap_left",
        right: "snap_right",
        up: "maximize_window",
        down: "minimize_window",
        click: "alt_tab",
    },
    GesturePreset {
        name: "Pan",
        left: "pan_left",
        right: "pan_right",
        up: "pan_up",
        down: "pan_down",
        click: "mouse_middle_click",
    },
    GesturePreset {
        name: "Zoom/Rotate",
        left: "rotate_left",
        right: "rotate_right",
        up: "zoom_in",
        down: "zoom_out",
        click: "zoom_reset",
    },
    GesturePreset {
        name: "App navigation",
        left: "alt_tab",
        right: "alt_tab",
        up: "start_menu",
        down: "win_d",
        click: "alt_tab",
    },
    GesturePreset {
        name: "Windows management",
        left: "snap_left",
        right: "snap_right",
        up: "maximize_window",
        down: "win_d",
        click: "alt_tab",
    },
    GesturePreset {
        name: "Media controls",
        left: "prev_track",
        right: "next_track",
        up: "volume_up",
        down: "volume_down",
        click: "play_pause",
    },
    GesturePreset {
        name: "Virtual desktops",
        left: "space_left",
        right: "space_right",
        up: "start_menu",
        down: "win_d",
        click: "task_view",
    },
];

pub fn action_id_to_slot_display_name(action_id: &str) -> std::borrow::Cow<'_, str> {
    if action_id.starts_with("custom:") {
        return std::borrow::Cow::Owned(action_id.strip_prefix("custom:").unwrap().to_uppercase());
    }
    match action_id {
        "none" => std::borrow::Cow::Borrowed("Do nothing"),
        "snap_left" => std::borrow::Cow::Borrowed("Snap left"),
        "snap_right" => std::borrow::Cow::Borrowed("Snap right"),
        "maximize_window" => std::borrow::Cow::Borrowed("Maximize window"),
        "minimize_window" => std::borrow::Cow::Borrowed("Minimize window"),
        "alt_tab" => std::borrow::Cow::Borrowed("Switch application"),
        "pan_left" | "pan_right" | "pan_up" | "pan_down" | "pan" => {
            std::borrow::Cow::Borrowed("Pan")
        }
        "mouse_middle_click" => std::borrow::Cow::Borrowed("Middle button"),
        "rotate_left" | "rotate_right" | "rotate" => std::borrow::Cow::Borrowed("Rotate"),
        "zoom_in" => std::borrow::Cow::Borrowed("Zoom in"),
        "zoom_out" => std::borrow::Cow::Borrowed("Zoom out"),
        "zoom_reset" => std::borrow::Cow::Borrowed("Zoom reset"),
        "start_menu" => std::borrow::Cow::Borrowed("Start menu"),
        "win_d" => std::borrow::Cow::Borrowed("Show/hide desktop"),
        "prev_track" => std::borrow::Cow::Borrowed("Previous"),
        "next_track" => std::borrow::Cow::Borrowed("Next"),
        "volume_up" => std::borrow::Cow::Borrowed("Volume up"),
        "volume_down" => std::borrow::Cow::Borrowed("Volume down"),
        "play_pause" => std::borrow::Cow::Borrowed("Play/Pause"),
        "space_left" => std::borrow::Cow::Borrowed("Desktop left"),
        "space_right" => std::borrow::Cow::Borrowed("Desktop right"),
        "task_view" => std::borrow::Cow::Borrowed("Task view"),
        _ => {
            if action_id.contains('_') {
                std::borrow::Cow::Owned(action_id.replace('_', " "))
            } else {
                std::borrow::Cow::Borrowed(action_id)
            }
        }
    }
}

pub const SLOT_ACTIONS: &[(&str, &str)] = &[
    ("none", "Do nothing"),
    ("snap_left", "Snap left"),
    ("snap_right", "Snap right"),
    ("maximize_window", "Maximize window"),
    ("minimize_window", "Minimize window"),
    ("alt_tab", "Switch application"),
    ("pan", "Pan"),
    ("mouse_middle_click", "Middle button"),
    ("rotate", "Rotate"),
    ("zoom_in", "Zoom in"),
    ("zoom_out", "Zoom out"),
    ("zoom_reset", "Zoom reset"),
    ("start_menu", "Start menu"),
    ("win_d", "Show/hide desktop"),
    ("prev_track", "Previous"),
    ("next_track", "Next"),
    ("volume_up", "Volume up"),
    ("volume_down", "Volume down"),
    ("play_pause", "Play/Pause"),
    ("space_left", "Desktop left"),
    ("space_right", "Desktop right"),
    ("task_view", "Task view"),
    ("custom", "Keyboard Shortcut"),
];

pub fn resolve_generic_slot_action(action: &str, dir: &str) -> String {
    match action {
        "pan" => match dir {
            "left" => "pan_left".to_string(),
            "right" => "pan_right".to_string(),
            "up" => "pan_up".to_string(),
            _ => "pan_down".to_string(),
        },
        "rotate" => match dir {
            "left" => "rotate_left".to_string(),
            _ => "rotate_right".to_string(),
        },
        _ => action.to_string(),
    }
}

pub fn get_generic_action_id(action_id: &str) -> &str {
    if action_id.starts_with("pan_") {
        "pan"
    } else if action_id.starts_with("rotate_") {
        "rotate"
    } else {
        action_id
    }
}
