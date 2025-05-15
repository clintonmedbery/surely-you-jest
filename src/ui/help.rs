use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::app::{App, AppView};

/// Creates the help bar text with keyboard controls based on the current view
pub fn create_help_bar(app: &App) -> Paragraph<'static> {
    let mut spans = Vec::new();
    
    // Different controls based on the current view
    let controls: Vec<(&str, &str)> = match app.view {
        AppView::TestList => vec![
            ("↑/↓", "Navigate"),
            ("PgUp/PgDn", "Page Up/Down"),
            ("→", "View File"),
            ("Enter", "Run Test"),
            ("q", "Quit"),
        ],
        
        AppView::TestDetail => vec![
            ("←", "Back to List"),
            ("Enter", "Run Test"),
            ("q", "Quit"),
        ],
        
        AppView::TestRunning => vec![
            ("←", "Back to List"),
            ("↑/↓", "Scroll"),
            ("PgUp/PgDn", "Scroll Faster"),
            ("Home/End", "Top/Bottom"),
            ("Enter", "Copy Command"),
            ("q", "Quit"),
        ],
    };
    
    // Create styled spans for each control
    for (i, (key, description)) in controls.iter().enumerate() {
        // Add separator after first item
        if i > 0 {
            spans.push(Span::raw(" │ "));
        }
        
        // Add key with highlighting
        spans.push(Span::styled(
            key.to_string(),  // Convert to owned String to fix borrowing issue
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        ));
        
        // Add description
        spans.push(Span::raw(format!(": {}", description)));
    }
    
    // Create the paragraph
    Paragraph::new(Line::from(spans))
        .style(Style::default().fg(Color::Gray))
}