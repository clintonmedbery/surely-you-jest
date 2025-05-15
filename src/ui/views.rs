use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use crate::app::App;

/// Renders the test list view
pub fn render_test_list(app: &mut App, frame: &mut Frame, area: Rect) {
    // Calculate how many items can be displayed in the visible area
    // Subtract 2 for the borders
    let visible_area_height = area.height.saturating_sub(2) as usize;
    
    // Update scroll position based on selection
    app.update_scroll(visible_area_height);
    
    if app.tests.is_empty() {
        // No tests found message
        frame.render_widget(
            Paragraph::new("No test files found."),
            area
        );
        return;
    } 
    
    // Determine which items to show based on scroll position
    let end_idx = (app.scroll_offset + visible_area_height).min(app.tests.len());
    let visible_tests = &app.tests[app.scroll_offset..end_idx];
    
    // Create styled Text with proper highlighting
    let mut text = Text::default();
    
    for (i, line) in visible_tests.iter().enumerate() {
        let absolute_index = i + app.scroll_offset;
        let is_selected = absolute_index == app.selected_index;
        
        // Create the prefix (arrow or space)
        let prefix = if is_selected {
            Span::styled("➤ ", Style::default().fg(Color::Yellow))
        } else {
            Span::raw("  ")
        };
        
        // Create the test name with proper styling
        let test_span = if is_selected {
            // Highlight selected item with bold yellow on blue background
            Span::styled(
                line, 
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            )
        } else {
            // Regular item
            Span::raw(line)
        };
        
        // Add the line to the text
        text.lines.push(Line::from(vec![prefix, test_span]));
    }
    
    // Append scroll indicator if needed
    if app.tests.len() > visible_area_height {
        let scroll_info = format!(
            "[{}/{}]", 
            app.selected_index + 1, 
            app.tests.len()
        );
        text.lines.push(Line::from(Span::styled(
            scroll_info,
            Style::default().fg(Color::Gray)
        )));
    }
    
    // Create a block for the list
    let list_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default());
    
    // Render the styled list
    let test_list = Paragraph::new(text)
        .block(list_block);
        
    frame.render_widget(test_list, area);
}

/// Renders the test detail view
pub fn render_test_detail(app: &mut App, frame: &mut Frame, area: Rect) {
    // Create a block for the test content
    let content_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default());
    
    // Create the paragraph with the test file content
    let content = Paragraph::new(app.current_test_content.clone())
        .block(content_block)
        .wrap(Wrap { trim: false });
    
    // Render the content
    frame.render_widget(content, area);
}

/// Renders the test running view with terminal-like output
pub fn render_test_running(app: &mut App, frame: &mut Frame, area: Rect) {
    // Split the area into sections: command at top, output below
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(3), // Command bar
            ratatui::layout::Constraint::Min(1),    // Terminal output
        ])
        .split(area);
        
    // Get the command from the current test
    let test_file = &app.tests[app.selected_index];
    let project_dir = &app.search_path;
    let command = format!("cd {} && npx jest {} --no-cache", project_dir, test_file);
    
    // Terminal title with command
    let command_block = Block::default()
        .title(" Command ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
        
    let command_text = if app.copied_command.is_some() {
        format!("{} [Copied ✓]", command)
    } else {
        format!("{} [Press Enter to copy]", command)
    };
    
    let command_paragraph = Paragraph::new(command_text)
        .block(command_block)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });
        
    frame.render_widget(command_paragraph, chunks[0]);
    
    // Terminal output area
    let terminal_block = Block::default()
        .title(" Terminal Output ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
        
    // Convert raw output to styled text with syntax highlighting
    let mut text = Text::default();
    
    // Process each line of output to apply color
    let visible_lines = chunks[1].height.saturating_sub(2) as usize; // Account for borders
    let lines: Vec<&str> = app.test_run_output.lines().collect();
    
    // Calculate visible range with scrolling
    let start_line = app.terminal_scroll.min(lines.len().saturating_sub(1));
    let end_line = (start_line + visible_lines).min(lines.len());
    
    // Add each visible line with appropriate styling
    for line in &lines[start_line..end_line] {
        let line_str = *line; // Dereference to get &str
        let styled_line = if line_str.contains("PASS") || line_str.contains("✓") {
            Line::from(Span::styled(line_str, Style::default().fg(Color::Green)))
        } else if line_str.contains("FAIL") || line_str.contains("×") || line_str.contains("Error:") {
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
                Style::default().fg(Color::Gray)
            )));
        }
        
        if text.lines.len() < visible_lines && end_line >= lines.len() {
            text.lines.push(Line::from(Span::styled(
                scroll_indicator,
                Style::default().fg(Color::Gray)
            )));
        }
    }
    
    let terminal_paragraph = Paragraph::new(text)
        .block(terminal_block)
        .wrap(Wrap { trim: false });
        
    frame.render_widget(terminal_paragraph, chunks[1]);
}