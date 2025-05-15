use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget},
};

/// Widget for displaying a scrollable list of test files
pub struct TestListWidget<'a> {
    /// Tests to display
    pub tests: &'a [String],
    /// Currently selected index
    pub selected_index: usize,
    /// First visible item index
    pub scroll_offset: usize,
}

impl<'a> TestListWidget<'a> {
    /// Create a new test list widget
    pub fn new(tests: &'a [String], selected_index: usize, scroll_offset: usize) -> Self {
        Self {
            tests,
            selected_index,
            scroll_offset,
        }
    }
    
    /// Calculate maximum visible items in the given area
    pub fn visible_items(&self, area: Rect) -> usize {
        area.height.saturating_sub(2) as usize // Subtract 2 for potential borders
    }
    
    /// Update the scroll position based on selection and visible area
    pub fn update_scroll(&mut self, visible_items: usize) {
        if self.tests.is_empty() {
            return;
        }
        
        // If selection is below visible area, scroll down
        if self.selected_index >= self.scroll_offset + visible_items {
            self.scroll_offset = self.selected_index - visible_items + 1;
        }
        
        // Ensure scroll doesn't go past the end
        let max_scroll = self.tests.len().saturating_sub(visible_items);
        self.scroll_offset = self.scroll_offset.min(max_scroll);
    }
}

impl<'a> Widget for TestListWidget<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        // Update the scroll position based on selection and visible area
        let visible_items = self.visible_items(area);
        self.update_scroll(visible_items);
        
        // Create a block for the list
        let block = Block::default()
            .title("Test Files")
            .borders(Borders::ALL);
        
        // Render the block first
        let inner_area = block.inner(area);
        block.render(area, buf);
        
        // If no tests, show a message and return
        if self.tests.is_empty() {
            Paragraph::new("No test files found.")
                .render(inner_area, buf);
            return;
        }
        
        // Calculate visible range
        let visible_area_height = self.visible_items(inner_area);
        let end_idx = (self.scroll_offset + visible_area_height).min(self.tests.len());
        let visible_tests = &self.tests[self.scroll_offset..end_idx];
        
        // Create styled text for the list
        let mut text = Text::default();
        
        for (i, line) in visible_tests.iter().enumerate() {
            let absolute_index = i + self.scroll_offset;
            let is_selected = absolute_index == self.selected_index;
            
            // Create the selector string (arrow or space) - keep it inside the box
            let selector = if is_selected { "▶ " } else { "  " };
            
            // Create the test name with proper styling
            let line_text = format!("{}{}", selector, line);
            let styled_line = if is_selected {
                // Highlight selected item with bold yellow on blue background
                Span::styled(
                    line_text,
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                )
            } else {
                // Regular item
                Span::raw(line_text)
            };
            
            // Add the line to the text
            text.lines.push(Line::from(styled_line));
        }
        
        // Append scroll indicator if needed
        if self.tests.len() > visible_area_height {
            let scroll_info = format!(
                "[{}/{}]", 
                self.selected_index + 1, 
                self.tests.len()
            );
            text.lines.push(Line::from(Span::styled(
                scroll_info,
                Style::default().fg(Color::Gray)
            )));
        }
        
        // Render the text inside the block's inner area
        Paragraph::new(text)
            .render(inner_area, buf);
    }
}