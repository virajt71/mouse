pub mod actions_ring;
pub mod customization;
pub mod empty_state;
pub mod select_connection;
pub mod settings;
pub mod top_bar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    EmptyState,
    SelectConnectionType,
    Settings,
    Customization,
}
