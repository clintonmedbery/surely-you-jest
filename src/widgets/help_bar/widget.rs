use ratatui::{
    prelude::*,
    widgets::{Paragraph, Widget},
};

/// Widget for displaying keyboard control help at the bottom of the screen
pub struct HelpBarWidget<'a> {
    /// Controls to display [("key", "description"), ...]
    pub controls: Vec<(&'a str, &'a str)>,
}

impl<'a> HelpBarWidget<'a> {
    /// Create a new help bar widget with the given controls
    pub fn new(controls: Vec<(&'a str, &'a str)>) -> Self {
        Self { controls }
    }

    /// Create a help bar for test list view
    pub fn for_test_list() -> Self {
        Self::new(vec![
            ("↑/↓", "Navigate"),
            ("PgUp/PgDn", "Page Up/Down"),
            ("Ctrl+→", "View File"),
            ("→", "View Tests"),
            ("Enter", "Run Test"),
            ("q", "Quit"),
        ])
    }

    /// Create a help bar for test detail view
    pub fn for_test_detail() -> Self {
        Self::new(vec![
            ("←", "Back to List"),
            ("Enter", "Run Test"),
            ("q", "Quit"),
        ])
    }

    /// Create a help bar for test terminal view
    pub fn for_test_terminal() -> Self {
        Self::new(vec![
            ("←", "Back to List"),
            ("→", "View Tests"),
            ("↑/↓", "Scroll"),
            ("PgUp/PgDn", "Scroll Faster"),
            ("Home/End", "Top/Bottom"),
            ("Enter", "View Tests/Copy"),
            ("q", "Quit"),
        ])
    }

    /// Create a help bar for test results view
    pub fn for_test_results() -> Self {
        Self::new(vec![
            ("←", "Back to Output"),
            ("↑/↓", "Select Test"),
            ("→/Enter", "Run Selected Test"),
            ("q", "Quit"),
        ])
    }
}

impl<'a> Widget for HelpBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut spans = Vec::new();

        // Create styled spans for each control
        for (i, (key, description)) in self.controls.iter().enumerate() {
            // Add separator after first item
            if i > 0 {
                spans.push(Span::raw(" │ "));
            }

            // Add key with highlighting
            spans.push(Span::styled(
                key.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));

            // Add description
            spans.push(Span::raw(format!(": {}", description)));
        }

        // Create and render the paragraph
        Paragraph::new(Line::from(spans))
            .style(Style::default().fg(Color::Gray))
            .render(area, buf);
    }
}
