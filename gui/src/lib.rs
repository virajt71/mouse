#![allow(
    clippy::derivable_impls,
    clippy::field_reassign_with_default,
    clippy::wildcard_in_or_patterns,
    clippy::unnecessary_cast
)]
pub mod app;
pub mod desktop_apps;
pub mod icon_loader;
pub mod theme;
pub mod translation;
pub mod updater;
pub mod views;
pub mod widgets;

pub use app::MouserApp;
pub use views::ActiveView;
