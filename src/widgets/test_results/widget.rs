use crate::app::state::TestInfo;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub struct TestResultsWidget<'a> {
    pub tests: &'a [TestInfo],
    pub selected_index: usize,
}

impl<'a> TestResultsWidget<'a> {
    pub fn new(tests: &'a [TestInfo], selected_index: usize) -> Self {
        Self {
            tests,
            selected_index,
        }
    }
}

impl<'a> Widget for TestResultsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Split area vertically: list of tests (left) and selected test details (right)
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Build a custom test list

        // Create a block for the test list
        let block = Block::default().title("Test Results").borders(Borders::ALL);

        // Render the block first and get inner area
        let inner_area = block.inner(horizontal_chunks[0]);
        block.render(horizontal_chunks[0], buf);

        // Create a custom list with an internal selection indicator
        let mut list_text = Text::default();

        for (idx, test) in self.tests.iter().enumerate() {
            let is_selected = idx == self.selected_index;

            // Create the selector string (arrow or space) - keep it inside the box
            let selector = if is_selected { "▶ " } else { "  " };

            let status = if test.passed { "✅ " } else { "❌ " };

            let time_str = match test.duration {
                Some(ms) => format!(" ({} ms)", ms),
                None => String::new(),
            };

            // Create combined line text
            let line_text = format!("{}{}{}{}", selector, status, test.name, time_str);

            // Style based on selection and pass/fail status
            let style = if is_selected {
                Style::default()
                    .fg(if test.passed {
                        Color::Green
                    } else {
                        Color::Red
                    })
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(if test.passed {
                    Color::Green
                } else {
                    Color::Red
                })
            };

            list_text
                .lines
                .push(Line::from(Span::styled(line_text, style)));
        }

        // Render our custom list
        Paragraph::new(list_text).render(inner_area, buf);

        // Render the details of the selected test if any
        if !self.tests.is_empty() && self.selected_index < self.tests.len() {
            let selected_test = &self.tests[self.selected_index];

            // Create formatted test details
            let status = if selected_test.passed {
                "Passed"
            } else {
                "Failed"
            };

            let time = match selected_test.duration {
                Some(ms) => format!("{} ms", ms),
                None => "Unknown".to_string(),
            };

            let header_text = format!(
                "Name: {}\nStatus: {}\nDuration: {}",
                selected_test.name, status, time
            );

            let error_text = if let Some(ref error) = selected_test.error {
                format!("\n\nError Details:\n{}", error)
            } else {
                String::new()
            };

            let full_text = format!("{}{}", header_text, error_text);

            // Create style based on pass/fail status
            let title_style = if selected_test.passed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            // Render the details
            let detail_block = Block::default()
                .title(format!("Test Details"))
                .title_style(title_style)
                .borders(Borders::ALL);

            Paragraph::new(full_text)
                .block(detail_block)
                .wrap(Wrap { trim: false })
                .render(horizontal_chunks[1], buf);
        } else {
            // No test selected
            let no_test_selected = Paragraph::new("No test selected")
                .block(Block::default().title("Test Details").borders(Borders::ALL))
                .alignment(Alignment::Center);

            no_test_selected.render(horizontal_chunks[1], buf);
        }
    }
}
