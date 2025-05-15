use ratatui::prelude::*;
use ratatui::widgets::{Widget, Paragraph};

pub struct HeaderWidget {
    pub title: String,
    pub subtitle: String,
}

impl Widget for HeaderWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(vec![
            Span::styled(self.title, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - "),
            Span::styled(self.subtitle, Style::default().add_modifier(Modifier::ITALIC)),
        ]);

        Paragraph::new(line).render(area, buf);
    }
}