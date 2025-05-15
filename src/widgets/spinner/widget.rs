use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Widget, Paragraph},
};

/// Animation styles for the spinner
#[derive(Debug, Clone, Copy)]
pub enum SpinnerStyle {
    Line,
    Dot,
    Box,
}

/// A spinner widget that displays an animation to indicate processing
pub struct SpinnerWidget {
    /// The label to display next to the spinner
    label: String,
    /// The animation style to use
    style: SpinnerStyle,
}

impl Default for SpinnerWidget {
    fn default() -> Self {
        Self {
            label: "Loading...".to_string(),
            style: SpinnerStyle::Line,
        }
    }
}

impl SpinnerWidget {
    /// Create a new spinner with the given label
    pub fn new<S: Into<String>>(label: S) -> Self {
        Self {
            label: label.into(),
            ..Self::default()
        }
    }
    
    /// Set the spinner style
    pub fn style(mut self, style: SpinnerStyle) -> Self {
        self.style = style;
        self
    }
    
    /// Get the current animation frame based on system time
    fn current_frame(&self) -> &str {
        // Use the current time to determine the frame
        let frames = self.get_frames();
        let current_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as usize;
        let frame_index = (current_millis / 150) % frames.len(); // 150ms per frame
        
        frames[frame_index]
    }
    
    /// Get the frames for the current animation style
    fn get_frames(&self) -> &[&str] {
        match self.style {
            SpinnerStyle::Line => &["-", "\\", "|", "/"],
            SpinnerStyle::Dot => &["⠋", "⠙", "⠸", "⠴", "⠦", "⠇"],
            SpinnerStyle::Box => &["◰", "◳", "◲", "◱"],
        }
    }
}

impl Widget for SpinnerWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Get current frame
        let spinner_frame = self.current_frame();
        
        // Create the spinner text
        let text = format!("{} {}", spinner_frame, self.label);
        
        // Render with a nice block
        Paragraph::new(text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Running Test "))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan))
            .render(area, buf);
    }
}