use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Widget for displaying the content of a test file
pub struct TestDetailWidget<'a> {
    /// Content to display
    pub content: &'a str,
}

impl<'a> TestDetailWidget<'a> {
    /// Create a new test detail widget
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }
}

impl<'a> Widget for TestDetailWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render the file content
        Paragraph::new(self.content)
            .block(Block::default().borders(Borders::NONE))
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}