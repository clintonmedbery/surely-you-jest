use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Widget for displaying test execution output with terminal-like styling
pub struct TestTerminalWidget<'a> {
    /// Command that was run
    pub command: &'a str,
    /// Output from the command
    pub output: &'a str,
    /// Scroll position in the output
    pub scroll_position: usize,
    /// Whether the command has been copied
    pub command_copied: bool,
}

impl<'a> TestTerminalWidget<'a> {
    /// Create a new terminal widget
    pub fn new(
        command: &'a str,
        output: &'a str,
        scroll_position: usize,
        command_copied: bool,
    ) -> Self {
        Self {
            command,
            output,
            scroll_position,
            command_copied,
        }
    }
}

impl<'a> Widget for TestTerminalWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Split the area into command and output sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Command bar
                Constraint::Min(1),    // Terminal output
            ])
            .split(area);

        // Render command area with copy status
        let command_text = if self.command_copied {
            format!("{} [Copied ✓]", self.command)
        } else {
            format!("{} [Press Enter to copy]", self.command)
        };

        Paragraph::new(command_text)
            .block(
                Block::default()
                    .title(" Command ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .render(chunks[0], buf);

        // Process and render terminal output
        let mut text = Text::default();

        // Calculate visible range
        let visible_lines = chunks[1].height.saturating_sub(2) as usize; // Account for borders
        let lines: Vec<&str> = self.output.lines().collect();

        let start_line = self.scroll_position.min(lines.len().saturating_sub(1));
        let end_line = (start_line + visible_lines).min(lines.len());

        // Add each visible line with appropriate styling
        for line in &lines[start_line..end_line] {
            let line_str = *line; // Dereference to get &str
            let styled_line = if line_str.contains("PASS") || line_str.contains("✓") {
                Line::from(Span::styled(line_str, Style::default().fg(Color::Green)))
            } else if line_str.contains("FAIL")
                || line_str.contains("×")
                || line_str.contains("Error:")
            {
                Line::from(Span::styled(line_str, Style::default().fg(Color::Red)))
            } else if line_str.starts_with("    at ") || line_str.contains("Stack:") {
                // Stack traces in dimmed white
                Line::from(Span::styled(line_str, Style::default().fg(Color::Gray)))
            } else if line_str.contains("Expected:") || line_str.contains("Received:") {
                // Expected/Received in yellow
                Line::from(Span::styled(line_str, Style::default().fg(Color::Yellow)))
            } else if line_str.contains("console.log") || line_str.contains("console.info") {
                // Console output in cyan
                Line::from(Span::styled(line_str, Style::default().fg(Color::Cyan)))
            } else if line_str.contains("warning") || line_str.contains("Warning:") {
                // Warnings in yellow
                Line::from(Span::styled(line_str, Style::default().fg(Color::Yellow)))
            } else {
                // Default color
                Line::from(line_str)
            };

            text.lines.push(styled_line);
        }

        // Add scroll indicator if needed
        if lines.len() > visible_lines {
            let scroll_percentage = if lines.len() <= visible_lines {
                100.0
            } else {
                (start_line as f64 / (lines.len().saturating_sub(visible_lines)) as f64) * 100.0
            };

            let scroll_indicator = format!(
                "Scroll: {:.0}% ({}/{} lines) [↑/↓: Navigate | PgUp/PgDn: Scroll faster]",
                scroll_percentage,
                start_line + 1,
                lines.len()
            );

            if end_line < lines.len() {
                text.lines.push(Line::from(Span::styled(
                    "↓ More lines below ↓",
                    Style::default().fg(Color::Gray),
                )));
            }

            if text.lines.len() < visible_lines && end_line >= lines.len() {
                text.lines.push(Line::from(Span::styled(
                    scroll_indicator,
                    Style::default().fg(Color::Gray),
                )));
            }
        }

        // Render the terminal output
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Terminal Output ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .wrap(Wrap { trim: false })
            .render(chunks[1], buf);
    }
}
