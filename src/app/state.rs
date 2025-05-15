use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};
use std::{io, path::PathBuf, process::{Command, Stdio}, sync::mpsc};
use crate::jest::test_runner::{self, TestResult};

/// The different views of the application.
#[derive(Debug, PartialEq)]
pub enum AppView {
    /// Viewing the list of test files
    TestList,
    /// Viewing the details of a single test file
    TestDetail,
    /// Running a test file
    TestRunning,
    /// Viewing individual test results
    TestResults,
}

/// Information about an individual test case
#[derive(Debug, Clone)]
pub struct TestInfo {
    /// The test name/description
    pub name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Any error details
    pub error: Option<String>,
    /// Duration of the test in ms
    pub duration: Option<u64>,
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
    /// Whether the test is currently loading
    pub test_loading: bool,
    /// Channel for receiving test run updates
    pub test_receiver: Option<mpsc::Receiver<TestResult>>,
    /// Individual test results parsed from output
    pub individual_tests: Vec<TestInfo>,
    /// Selected test result index
    pub selected_test_index: usize,
    /// Flag to automatically show test results when test completes
    pub auto_show_test_results: bool,
    /// Flag indicating if we're running an individual test (vs a full file)
    pub running_individual_test: bool,
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
            test_loading: false,
            test_receiver: None,
            individual_tests: Vec::new(),
            selected_test_index: 0,
            auto_show_test_results: false,
            running_individual_test: false,
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
            test_loading: false,
            test_receiver: None,
            individual_tests: Vec::new(),
            selected_test_index: 0,
            auto_show_test_results: false,
            running_individual_test: false,
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

    // We no longer need the update_scroll method as this is now managed by TestListWidget
    
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
        self.test_loading = true;
        self.test_run_output = String::new(); // Clear previous output
        self.running_individual_test = false; // Flag that we're running a full test file
        
        // Need to clone these for the async task
        let test_file = self.tests[self.selected_index].clone();
        let project_dir = self.search_path.clone();
        
        // Start the async test process
        self.test_receiver = Some(test_runner::start_async_test(&test_file, &project_dir));
        
        // Show initial "running test" message
        self.test_run_output = format!("Running test: {}\n", test_file);
        
        Ok(())
    }
    
    /// Navigate back based on context
    pub fn go_back(&mut self) {
        if self.view == AppView::TestRunning && self.running_individual_test && !self.individual_tests.is_empty() {
            // If we're running an individual test, go back to test results view
            self.view = AppView::TestResults;
            self.terminal_scroll = 0;
            self.copied_command = None;
            self.running_individual_test = false;
        } else {
            // Otherwise go back to the test list
            self.view = AppView::TestList;
            self.terminal_scroll = 0;
            self.copied_command = None;
            self.running_individual_test = false;
        }
    }
    
    /// Compatibility method - delegates to go_back
    pub fn back_to_list(&mut self) {
        self.go_back();
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
    
    /// Parse individual test results from Jest output
    pub fn parse_test_results(&mut self) {
        self.individual_tests.clear();
        
        let mut current_test_name = String::new();
        let mut current_test_passed = false;
        let mut current_test_error = None;
        let mut current_test_duration = None;
        
        // Process each line to find test results
        for line in self.test_run_output.lines() {
            let line = line.trim();
            
            // Check for test start
            if line.starts_with("✓") || line.starts_with("PASS") {
                // Test passed
                if !current_test_name.is_empty() {
                    // Save previous test if we have one
                    self.individual_tests.push(TestInfo {
                        name: current_test_name.clone(),
                        passed: true,
                        error: None,
                        duration: current_test_duration,
                    });
                }
                
                // Extract test name - remove the "✓ " prefix and extract the name
                let name_parts: Vec<&str> = line.splitn(2, ' ').collect();
                if name_parts.len() > 1 {
                    current_test_name = name_parts[1].trim().to_string();
                    current_test_passed = true;
                    current_test_error = None;
                    
                    // Try to extract duration if it's in the format "name (Duration: 10ms)"
                    if let Some(duration_idx) = current_test_name.find("(") {
                        if let Some(end_idx) = current_test_name.find(")") {
                            let duration_str = &current_test_name[duration_idx + 1..end_idx];
                            if duration_str.contains("ms") {
                                // Extract number from string like "10ms"
                                if let Some(ms_idx) = duration_str.find("ms") {
                                    let number_str = &duration_str[0..ms_idx].trim();
                                    if let Ok(duration) = number_str.parse::<u64>() {
                                        current_test_duration = Some(duration);
                                    }
                                }
                            }
                            // Remove the duration part from the name
                            current_test_name = current_test_name[0..duration_idx].trim().to_string();
                        }
                    }
                }
            } else if line.starts_with("×") || line.starts_with("FAIL") {
                // Test failed
                if !current_test_name.is_empty() {
                    // Save previous test if we have one
                    self.individual_tests.push(TestInfo {
                        name: current_test_name.clone(),
                        passed: current_test_passed,
                        error: current_test_error.clone(),
                        duration: current_test_duration,
                    });
                }
                
                // Extract test name - remove the "× " prefix and extract the name
                let name_parts: Vec<&str> = line.splitn(2, ' ').collect();
                if name_parts.len() > 1 {
                    current_test_name = name_parts[1].trim().to_string();
                    current_test_passed = false;
                    current_test_error = Some(String::new()); // Will be populated with subsequent error lines
                    current_test_duration = None;
                    
                    // Try to extract duration
                    if let Some(duration_idx) = current_test_name.find("(") {
                        if let Some(end_idx) = current_test_name.find(")") {
                            let duration_str = &current_test_name[duration_idx + 1..end_idx];
                            if duration_str.contains("ms") {
                                // Extract number from string like "10ms"
                                if let Some(ms_idx) = duration_str.find("ms") {
                                    let number_str = &duration_str[0..ms_idx].trim();
                                    if let Ok(duration) = number_str.parse::<u64>() {
                                        current_test_duration = Some(duration);
                                    }
                                }
                            }
                            // Remove the duration part from the name
                            current_test_name = current_test_name[0..duration_idx].trim().to_string();
                        }
                    }
                }
            } else if !current_test_name.is_empty() && line.contains("Error:") && !current_test_passed {
                // Found error details
                if let Some(ref mut error) = current_test_error {
                    error.push_str(line);
                    error.push('\n');
                }
            }
        }
        
        // Add the last test if there is one
        if !current_test_name.is_empty() {
            self.individual_tests.push(TestInfo {
                name: current_test_name,
                passed: current_test_passed,
                error: current_test_error,
                duration: current_test_duration,
            });
        }
        
        // Reset selection
        self.selected_test_index = 0;
    }
    
    /// View individual test results from test output
    pub fn view_test_results(&mut self) {
        if self.view == AppView::TestRunning && !self.test_loading {
            // Parse the results first
            self.parse_test_results();
            
            // Only switch view if we found some tests
            if !self.individual_tests.is_empty() {
                self.view = AppView::TestResults;
                self.selected_test_index = 0;
            }
        }
    }
    
    /// Load and parse test file to extract individual tests without running them
    pub fn load_and_parse_individual_tests(&mut self) -> io::Result<()> {
        if self.tests.is_empty() {
            return Ok(());
        }
        
        // Clear any previous test results
        self.individual_tests.clear();
        
        // Get the test file path
        let test_file = &self.tests[self.selected_index];
        let full_path = PathBuf::from(&self.search_path).join(test_file);
        
        // Read the file content
        let content = std::fs::read_to_string(&full_path)?;
        
        // Parse the file to find test definitions
        self.parse_test_definitions(&content);
        
        Ok(())
    }
    
    /// Parse test content to extract individual test definitions
    pub fn parse_test_definitions(&mut self, content: &str) {
        self.individual_tests.clear();
        
        // Common Jest/Testing Library test patterns
        let test_patterns = [
            // Jest/Testing Library pattern: test('description', () => {});
            r#"(?:test|it)\s*\(\s*['"](.+?)['"]"#,
            // Describe blocks: describe('description', () => {});
            r#"describe\s*\(\s*['"](.+?)['"]"#,
        ];
        
        // Process each line to find test definitions
        let mut in_comment_block = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines
            if line.is_empty() {
                continue;
            }
            
            // Handle comment blocks
            if line.starts_with("/*") {
                in_comment_block = true;
            }
            if line.contains("*/") {
                in_comment_block = false;
                continue;
            }
            if in_comment_block || line.starts_with("//") {
                continue;
            }
            
            // Look for test patterns
            for pattern in &test_patterns {
                // Create a regex for the pattern
                if let Ok(re) = regex::Regex::new(pattern) {
                    // Find all matches in the line
                    for cap in re.captures_iter(line) {
                        if let Some(description) = cap.get(1) {
                            let description = description.as_str().trim().to_string();
                            
                            // Skip if we already have this test
                            if self.individual_tests.iter().any(|t| t.name == description) {
                                continue;
                            }
                            
                            // Add the test to our list
                            self.individual_tests.push(TestInfo {
                                name: description,
                                passed: false, // We don't know yet
                                error: None,
                                duration: None,
                            });
                        }
                    }
                }
            }
        }
        
        // Reset the selection index
        self.selected_test_index = 0;
    }
    
    /// Run an individual test using Jest's testNamePattern option
    pub fn run_individual_test(&mut self) -> io::Result<()> {
        if self.individual_tests.is_empty() || self.selected_test_index >= self.individual_tests.len() {
            return Ok(());
        }
        
        // Get the currently selected test
        let selected_test = &self.individual_tests[self.selected_test_index];
        let test_name = &selected_test.name;
        
        // Set up state for test running
        self.view = AppView::TestRunning;
        self.test_loading = true;
        self.test_run_output = String::new(); // Clear previous output
        self.running_individual_test = true; // Flag that we're running an individual test
        
        // Get the file path
        let test_file = if !self.tests.is_empty() {
            self.tests[self.selected_index].clone()
        } else {
            return Ok(());
        };
        
        // Create a thread to run the specific test
        let test_name_pattern = test_name.replace("\"", "\\\""); // Escape quotes for shell
        let test_name_pattern_clone = test_name_pattern.clone(); // Clone for use in closure
        let test_file_clone = test_file.clone();
        let project_dir = self.search_path.clone();
        
        // Create a channel to receive the results 
        let (tx, rx) = mpsc::channel();
        
        // Spawn a thread to run the test
        std::thread::spawn(move || {
            // Execute the Jest test with testNamePattern option
            let output = Command::new("npx")
                .args([
                    "jest", 
                    &test_file_clone, 
                    "--no-cache",
                    "--testNamePattern", 
                    &format!("^{}$", test_name_pattern_clone), // Exact match pattern
                ])
                .current_dir(PathBuf::from(&project_dir))
                .output();
            
            // Send the running signal first
            let _ = tx.send(TestResult::Running);
            
            // Then send the completed result
            let result = match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    Ok((stdout, stderr))
                },
                Err(e) => Err(e)
            };
            
            let _ = tx.send(TestResult::Completed(result));
        });
        
        // Store the channel
        self.test_receiver = Some(rx);
        
        // Show initial "running test" message with command info
        self.test_run_output = format!(
            "Running individual test: \"{}\"\nFile: {}\nCommand: npx jest {} --testNamePattern=\"^{}$\" --no-cache\n",
            test_name,
            test_file,
            test_file,
            test_name_pattern
        );
        
        Ok(())
    }

    /// Check for test results from the async runner
    pub fn check_test_results(&mut self) {
        // If we have a receiver and we're in the test running state
        if let Some(receiver) = &self.test_receiver {
            // Try to receive a message without blocking
            match receiver.try_recv() {
                Ok(TestResult::Running) => {
                    // Test is still running, keep the loading state
                    self.test_loading = true;
                },
                Ok(TestResult::Completed(result)) => {
                    // Test is complete, process the result
                    self.test_loading = false;
                    
                    match result {
                        Ok((stdout, stderr)) => {
                            // Store raw command and output for display in TUI
                            self.test_run_output = format!("{}\n{}", stdout, stderr);
                        },
                        Err(e) => {
                            // Simple error message
                            self.test_run_output = format!("Error running test: {}", e);
                        }
                    }
                    
                    // We're done with this receiver
                    self.test_receiver = None;
                    
                    // Calculate appropriate scroll position to show last line at the bottom
                    // First, get a rough estimate of the visible height (we won't know exact until render)
                    let approx_visible_lines = 20; // Reasonable estimate for most terminals
                    let line_count = self.test_run_output.lines().count();
                    
                    // Set scroll position to show the last page of output
                    // This puts the last line at the bottom of the window instead of the top
                    self.terminal_scroll = line_count.saturating_sub(approx_visible_lines).max(0);
                    
                    // If auto_show_test_results is enabled, try to parse and show individual tests
                    if self.auto_show_test_results {
                        self.auto_show_test_results = false; // Reset the flag
                        
                        // Parse and show test results if available
                        self.parse_test_results();
                        if !self.individual_tests.is_empty() {
                            self.view = AppView::TestResults;
                            self.selected_test_index = 0;
                        }
                    }
                },
                Err(mpsc::TryRecvError::Empty) => {
                    // No message yet, keep waiting
                },
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Channel closed, reset state
                    self.test_loading = false;
                    if self.test_run_output.is_empty() {
                        self.test_run_output = "Test execution failed or was cancelled".to_string();
                    }
                    self.test_receiver = None;
                }
            }
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        
        // Track the last time we rendered to enforce a minimum frame rate for animations
        let mut last_render = std::time::Instant::now();
        
        while self.running {
            // Check for test updates
            self.check_test_results();
            
            // Calculate time since last render
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_render);
            
            // If we're in loading state or enough time has passed, redraw
            if self.test_loading || elapsed > std::time::Duration::from_millis(100) {
                // Draw the UI
                terminal.draw(|frame| self.render(frame))?;
                last_render = now;
            }
            
            // Use a shorter timeout while loading to keep animation smooth
            let poll_timeout = if self.test_loading {
                std::time::Duration::from_millis(16) // ~60fps for smooth animation
            } else {
                std::time::Duration::from_millis(100)
            };
            
            // Handle user input with a timeout
            if event::poll(poll_timeout)? {
                self.handle_crossterm_events()?;
            }
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
                
                // View test file content (Ctrl+Right arrow)
                (KeyModifiers::CONTROL, KeyCode::Right) => {
                    if !self.tests.is_empty() {
                        let _ = self.load_test_content();
                    }
                },
                
                // View file content and parse tests (right arrow)
                (_, KeyCode::Right) => {
                    if !self.tests.is_empty() {
                        // First, load the test file content to parse
                        let _ = self.load_and_parse_individual_tests();
                        
                        // If we found tests, show the test results view
                        if !self.individual_tests.is_empty() {
                            self.view = AppView::TestResults;
                        }
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
                
                // Go back (left arrow)
                (_, KeyCode::Left) => self.go_back(),
                
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
                
                // Go back (left arrow)
                (_, KeyCode::Left) => self.go_back(),
                
                // View individual test results (right arrow)
                (_, KeyCode::Right) => {
                    if !self.test_loading {
                        self.view_test_results();
                    }
                },
                
                // Copy command to clipboard (Enter)
                (_, KeyCode::Enter) => {
                    if !self.test_loading {
                        // Parse tests first to see if we have any
                        self.parse_test_results();
                        
                        // If we have tests, view them, otherwise copy command
                        if !self.individual_tests.is_empty() {
                            self.view_test_results();
                        } else {
                            let _ = self.copy_command_to_clipboard();
                        }
                    }
                },
                
                // Scrolling for terminal output
                (_, KeyCode::Up) => self.scroll_up(1),
                (_, KeyCode::Down) => self.scroll_down(1),
                (_, KeyCode::PageUp) => self.scroll_up(10),
                (_, KeyCode::PageDown) => self.scroll_down(10),
                (_, KeyCode::Home) => self.terminal_scroll = 0,
                (_, KeyCode::End) => {
                    // Set scroll position to show the last page of output with last line at bottom
                    let approx_visible_lines = 20; // Reasonable estimate for most terminals
                    let line_count = self.test_run_output.lines().count();
                    self.terminal_scroll = line_count.saturating_sub(approx_visible_lines).max(0);
                },
                
                // Ignore other keys
                _ => {}
            },
            
            AppView::TestResults => match (key.modifiers, key.code) {
                // Exit application
                (_, KeyCode::Esc | KeyCode::Char('q'))
                | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
                
                // Back to test output view (left arrow)
                (_, KeyCode::Left) => {
                    // If we came from running a test, go back to running view
                    // otherwise go back to the list
                    if self.test_run_output.trim().is_empty() {
                        self.view = AppView::TestList;
                    } else {
                        self.view = AppView::TestRunning;
                    }
                },
                
                // Run individual test (right arrow or Enter)
                (_, KeyCode::Right | KeyCode::Enter) => {
                    if !self.individual_tests.is_empty() {
                        let _ = self.run_individual_test();
                    }
                },
                
                // Navigation of test results
                (_, KeyCode::Up | KeyCode::Char('k')) => {
                    if !self.individual_tests.is_empty() {
                        self.selected_test_index = self.selected_test_index.saturating_sub(1);
                    }
                },
                (_, KeyCode::Down | KeyCode::Char('j')) => {
                    if !self.individual_tests.is_empty() {
                        self.selected_test_index = (self.selected_test_index + 1)
                            .min(self.individual_tests.len().saturating_sub(1));
                    }
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
        use crate::widgets::{HeaderWidget, TestListWidget, TestDetailWidget, TestTerminalWidget, TestResultsWidget, HelpBarWidget, SpinnerWidget};
        
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
            },
            AppView::TestResults => {
                let test_name = if !self.tests.is_empty() {
                    &self.tests[self.selected_index]
                } else {
                    "Unknown Test"
                };
                (
                    "Individual Tests".to_string(),
                    format!("File: {}", test_name)
                )
            }
        };

        // Render the header widget at the top
        frame.render_widget(
            HeaderWidget {
                title,
                subtitle,
            },
            chunks[0],
        );

        // Render appropriate content based on the current view
        match self.view {
            AppView::TestList => {
                let widget = TestListWidget::new(
                    &self.tests,
                    self.selected_index,
                    self.scroll_offset
                );
                frame.render_widget(widget, chunks[1]);
            },
            AppView::TestDetail => {
                let widget = TestDetailWidget::new(&self.current_test_content);
                frame.render_widget(widget, chunks[1]);
            },
            AppView::TestRunning => {
                // Get command for the currently selected test
                let test_file = if !self.tests.is_empty() {
                    &self.tests[self.selected_index]
                } else {
                    ""
                };
                let command = format!("cd {} && npx jest {} --no-cache", self.search_path, test_file);
                
                if self.test_loading {
                    // Show spinner when test is loading
                    let test_name = if !self.tests.is_empty() {
                        &self.tests[self.selected_index]
                    } else {
                        "test"
                    };
                    let spinner = SpinnerWidget::new(format!("Running {}...", test_name))
                        .style(crate::widgets::spinner::SpinnerStyle::Dot);
                    
                    // Center the spinner in the content area
                    let spinner_area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(40),
                            Constraint::Length(3),
                            Constraint::Percentage(40),
                        ])
                        .split(chunks[1])[1];
                    
                    frame.render_widget(spinner, spinner_area);
                } else {
                    // Show test results when loading is complete
                    let widget = TestTerminalWidget::new(
                        &command,
                        &self.test_run_output,
                        self.terminal_scroll,
                        self.copied_command.is_some()
                    );
                    frame.render_widget(widget, chunks[1]);
                }
            },
            AppView::TestResults => {
                let widget = TestResultsWidget::new(
                    &self.individual_tests,
                    self.selected_test_index
                );
                frame.render_widget(widget, chunks[1]);
            }
        }
        
        // Render the appropriate help bar for the current view
        let help_bar = match self.view {
            AppView::TestList => HelpBarWidget::for_test_list(),
            AppView::TestDetail => HelpBarWidget::for_test_detail(),
            AppView::TestRunning => HelpBarWidget::for_test_terminal(),
            AppView::TestResults => HelpBarWidget::for_test_results(),
        };
        frame.render_widget(help_bar, chunks[2]);
    }
}