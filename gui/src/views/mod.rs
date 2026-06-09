pub mod customization;
pub mod select_connection;
pub mod top_bar;
pub mod settings;
pub mod empty_state;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    EmptyState,
    SelectConnectionType,
    Settings,
    Customization,
}
