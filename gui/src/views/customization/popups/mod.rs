pub mod action_list;
pub mod add_app_modal;
pub mod gesture_config;
pub mod record_shortcut;
pub mod thumbwheel;

pub use action_list::*;
pub use add_app_modal::*;
pub use gesture_config::*;
pub use record_shortcut::*;
pub use thumbwheel::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupView {
    ActionList,
    GesturesConfig,
    RecordShortcut {
        target_key: String,
        display_label: String,
    },
}
