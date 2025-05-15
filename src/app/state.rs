use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};
use std::{io, path::PathBuf, process::{Command, Stdio}};
use crate::ui::help::create_help_bar;
use crate::jest::test_runner;

/// The different views of the application.
#[derive(Debug, PartialEq)]
pub enum AppView {
    /// Viewing the list of test files
    TestList,
    /// Viewing the details of a single test file
    TestDetail,
    /// Running a test file
    TestRunning,
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// The path being searched for tests
    pub search_path: String,
    /// The testMatch patterns being used to find tests
    pub test_matches: Vec<String>,
    /// All the tests that are found in the search path
    pub tests: Vec<String>,
    /// Current selected index in the list
    pub selected_index: usize,
    /// First visible item in the scrolling list
    pub scroll_offset: usize,
    /// Current view state (list, detail, running)
    pub view: AppView,
    /// Content of the currently selected test file
    pub current_test_content: String,
    /// Status of the most recent test run
    pub test_run_output: String,
    /// Terminal output scroll position
    pub terminal_scroll: usize,
    /// Command that was copied to clipboard
    pub copied_command: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: false,
            search_path: String::new(),
            test_matches: Vec::new(),
            tests: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            view: AppView::TestList,
            current_test_content: String::new(),
            test_run_output: String::new(),
            terminal_scroll: 0,
            copied_command: None,
        }
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(search_path: String, test_matches: Vec<String>, tests: Vec<String>) -> Self {
        Self {
            running: false,
            search_path,
            test_matches,
            tests,
            selected_index: 0,
            scroll_offset: 0,
            view: AppView::TestList,
            current_test_content: String::new(),
            test_run_output: String::new(),
            terminal_scroll: 0,
            copied_command: None,
        }
    }
    
    /// Move selection up in the list
    pub fn previous(&mut self) {
        if !self.tests.is_empty() {
            self.selected_index = self.selected_index.saturating_sub(1);
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }

    /// Move selection down in the list
    pub fn next(&mut self) {
        if !self.tests.is_empty() {
            let last_index = self.tests.len() - 1;
            self.selected_index = (self.selected_index + 1).min(last_index);
        }
    }

    /// Adjust scroll offset based on current selection and visible area
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
    
    /// Load the content of the currently selected test file
    pub fn load_test_content(&mut self) -> io::Result<()> {
        if self.tests.is_empty() {
            return Ok(());
        }
        
        let test_file = &self.tests[self.selected_index];
        let full_path = PathBuf::from(&self.search_path).join(test_file);
        
        match std::fs::read_to_string(&full_path) {
            Ok(content) => {
                self.current_test_content = content;
                self.view = AppView::TestDetail;
                Ok(())
            },
            Err(e) => {
                self.current_test_content = format!("Error reading file: {}", e);
                self.view = AppView::TestDetail;
                Err(e)
            }
        }
    }
    
    /// Run the currently selected test file with Jest
    pub fn run_test(&mut self) -> io::Result<()> {
        if self.tests.is_empty() {
            return Ok(());
        }
        
        self.view = AppView::TestRunning;
        
        let test_file = &self.tests[self.selected_index];
        
        // Use the project root directory for the command
        let project_dir = &self.search_path;
        
        // Reset terminal scroll position
        self.terminal_scroll = 0;
        
        // Run the test with Jest
        let result = test_runner::run_jest_test(test_file, project_dir);
        
        match result {
            Ok((stdout, stderr)) => {
                // Store raw command and output for display in TUI
                self.test_run_output = format!("{}\n{}", stdout, stderr);
                self.copied_command = None; // Reset copied state
                
                Ok(())
            },
            Err(e) => {
                // Simple error message
                self.test_run_output = format!("Error running test: {}", e);
                self.copied_command = None;
                Err(e)
            }
        }
    }
    
    /// Return to the list view
    pub fn back_to_list(&mut self) {
        self.view = AppView::TestList;
        self.terminal_scroll = 0;
        self.copied_command = None;
    }
    
    /// Scroll terminal output up
    pub fn scroll_up(&mut self, amount: usize) {
        if self.view == AppView::TestRunning {
            self.terminal_scroll = self.terminal_scroll.saturating_sub(amount);
        }
    }
    
    /// Scroll terminal output down
    pub fn scroll_down(&mut self, amount: usize) {
        if self.view == AppView::TestRunning {
            // Count lines in output to determine max scroll
            let line_count = self.test_run_output.lines().count();
            self.terminal_scroll = (self.terminal_scroll + amount).min(line_count.saturating_sub(1));
        }
    }
    
    /// Copy the test command to the clipboard
    pub fn copy_command_to_clipboard(&mut self) -> io::Result<()> {
        if self.tests.is_empty() || self.view != AppView::TestRunning {
            return Ok(());
        }
        
        let test_file = &self.tests[self.selected_index];
        
        // Use the project root directory (search_path) rather than the test file's directory
        let project_dir = &self.search_path;
        
        // Build the shell command - cd to project root, then run Jest with relative test path
        let shell_command = format!(
            "cd {} && npx jest {} --no-cache", 
            project_dir,
            test_file  // Use relative path from project root
        );
        
        // Use pbcopy on macOS to copy to clipboard
        let copy_result = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.as_mut().unwrap().write_all(shell_command.as_bytes())?;
                child.wait().map(|_| ())
            });
            
        match copy_result {
            Ok(_) => {
                // Store the command that was copied
                self.copied_command = Some(shell_command.clone());
                Ok(())
            },
            Err(e) => {
                // Mark that copy failed
                self.copied_command = None;
                Err(e)
            }
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match self.view {
            AppView::TestList => match (key.modifiers, key.code) {
                // Exit application
                (_, KeyCode::Esc | KeyCode::Char('q'))
                | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
                
                // Navigation keys
                (_, KeyCode::Up | KeyCode::Char('k')) => self.previous(),
                (_, KeyCode::Down | KeyCode::Char('j')) => self.next(),
                
                // Page up/down for faster navigation
                (_, KeyCode::PageUp) => {
                    for _ in 0..10 {
                        self.previous();
                    }
                },
                (_, KeyCode::PageDown) => {
                    for _ in 0..10 {
                        self.next();
                    }
                },
                
                // Home/End to jump to beginning/end
                (_, KeyCode::Home) => {
                    self.selected_index = 0;
                    self.scroll_offset = 0;
                },
                (_, KeyCode::End) => {
                    if !self.tests.is_empty() {
                        self.selected_index = self.tests.len() - 1;
                    }
                },
                
                // View test details (right arrow)
                (_, KeyCode::Right) => {
                    if !self.tests.is_empty() {
                        let _ = self.load_test_content();
                    }
                },
                
                // Run test (enter/return)
                (_, KeyCode::Enter) => {
                    if !self.tests.is_empty() {
                        let _ = self.run_test();
                    }
                },
                
                // Ignore other keys
                _ => {}
            },
            
            AppView::TestDetail => match (key.modifiers, key.code) {
                // Exit application
                (_, KeyCode::Esc | KeyCode::Char('q'))
                | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
                
                // Back to list view (left arrow)
                (_, KeyCode::Left) => self.back_to_list(),
                
                // Run test (enter/return)
                (_, KeyCode::Enter) => {
                    let _ = self.run_test();
                },
                
                // Ignore other keys
                _ => {}
            },
            
            AppView::TestRunning => match (key.modifiers, key.code) {
                // Exit application
                (_, KeyCode::Esc | KeyCode::Char('q'))
                | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
                
                // Back to list view (left arrow)
                (_, KeyCode::Left) => self.back_to_list(),
                
                // Copy command to clipboard (right arrow or Enter)
                (_, KeyCode::Right | KeyCode::Enter) => {
                    let _ = self.copy_command_to_clipboard();
                },
                
                // Scrolling for terminal output
                (_, KeyCode::Up) => self.scroll_up(1),
                (_, KeyCode::Down) => self.scroll_down(1),
                (_, KeyCode::PageUp) => self.scroll_up(10),
                (_, KeyCode::PageDown) => self.scroll_down(10),
                (_, KeyCode::Home) => self.terminal_scroll = 0,
                (_, KeyCode::End) => {
                    let line_count = self.test_run_output.lines().count();
                    self.terminal_scroll = line_count.saturating_sub(1);
                },
                
                // Ignore other keys
                _ => {}
            },
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
    
    /// Renders the user interface
    pub fn render(&mut self, frame: &mut Frame) {
        use crate::ui::views::{render_test_list, render_test_detail, render_test_running};
        
        let area = frame.area();

        // Split the screen vertically: header (3 lines), main content, help bar (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),     // Header
                Constraint::Min(0),        // Main content
                Constraint::Length(1),     // Help bar
            ])
            .split(area);

        // Block fills the entire screen
        let block = Block::default()
            .title("Surely You Jest")
            .borders(Borders::ALL);
        frame.render_widget(block, area);

        // Determine the appropriate title and subtitle based on the current view
        let (title, subtitle) = match self.view {
            AppView::TestList => (
                "Surely You Jest".to_string(),
                format!(
                    "Tests in: {} (Found: {}) [Patterns: {}]", 
                    self.search_path, 
                    self.tests.len(),
                    self.test_matches.join(", ")
                )
            ),
            AppView::TestDetail => {
                let test_name = if !self.tests.is_empty() {
                    &self.tests[self.selected_index]
                } else {
                    "Unknown Test"
                };
                (
                    "Test File".to_string(),
                    format!("{}", test_name)
                )
            },
            AppView::TestRunning => {
                let test_name = if !self.tests.is_empty() {
                    &self.tests[self.selected_index]
                } else {
                    "Unknown Test"
                };
                (
                    "Test Results".to_string(),
                    format!("Running: {}", test_name)
                )
            }
        };

        // Render the header widget at the top
        frame.render_widget(
            crate::widgets::HeaderWidget {
                title,
                subtitle,
            },
            chunks[0],
        );

        // Render appropriate content based on the current view
        match self.view {
            AppView::TestList => render_test_list(self, frame, chunks[1]),
            AppView::TestDetail => render_test_detail(self, frame, chunks[1]),
            AppView::TestRunning => render_test_running(self, frame, chunks[1]),
        }
        
        // Render the appropriate help bar for the current view
        let help_text = create_help_bar(self);
        frame.render_widget(help_text, chunks[2]);
    }
}