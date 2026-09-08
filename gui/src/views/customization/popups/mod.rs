pub mod action_list;
pub mod add_app_modal;
pub mod gesture_config;
pub mod record_shortcut;
pub mod ring_action_picker;
pub mod ring_folder_popup;
pub mod thumbwheel;

pub use action_list::*;
pub use add_app_modal::*;
pub use gesture_config::*;
pub use record_shortcut::*;
pub use ring_action_picker::*;
pub use ring_folder_popup::*;
pub use thumbwheel::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupView {
    ActionList,
    GesturesConfig,
    RecordShortcut {
        target_key: String,
        display_label: String,
    },
    RingActionPicker {
        bubble_index: usize,
    },
    RingFolderPopup {
        folder_index: usize,
    },
    RingFolderBubblePicker {
        folder_index: usize,
        bubble_index: usize,
    },
}
