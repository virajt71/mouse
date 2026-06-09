pub mod action_list;
pub mod gesture_config;
pub mod record_shortcut;
pub mod thumbwheel;
pub mod add_app_modal;

pub use action_list::*;
pub use gesture_config::*;
pub use record_shortcut::*;
pub use thumbwheel::*;
pub use add_app_modal::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupView {
    ActionList,
    GesturesConfig,
    RecordShortcut { target_key: String, display_label: String },
}
