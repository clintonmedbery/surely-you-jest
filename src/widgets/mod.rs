// Basic widgets
pub mod header;
pub mod spinner;

// View-specific widgets
pub mod help_bar;
pub mod test_detail;
pub mod test_list;
pub mod test_results;
pub mod test_terminal;

// Re-export widgets for easy access
pub use header::HeaderWidget;
pub use help_bar::HelpBarWidget;
pub use spinner::SpinnerWidget;
pub use test_detail::TestDetailWidget;
pub use test_list::TestListWidget;
pub use test_results::TestResultsWidget;
pub use test_terminal::TestTerminalWidget;
