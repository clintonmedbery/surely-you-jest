use std::path::PathBuf;
use std::io;
use std::process::Command;
use std::sync::mpsc;

/// Runs a Jest test and returns the stdout and stderr output
pub fn run_jest_test(test_file: &str, project_dir: &str) -> io::Result<(String, String)> {
    // Execute the command from the project directory
    let output = Command::new("npx")
        .args(["jest", test_file, "--no-cache"])  // Use relative path 
        .current_dir(PathBuf::from(project_dir))  // Run from project directory
        .output()?;
    
    // Extract stdout and stderr
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    Ok((stdout, stderr))
}

/// Result of a test run
pub enum TestResult {
    /// Test is still running
    Running,
    /// Test has completed
    Completed(io::Result<(String, String)>),
}

/// Starts an async test run and returns a channel to receive updates
pub fn start_async_test(test_file: &str, project_dir: &str) -> mpsc::Receiver<TestResult> {
    let test_file = test_file.to_string();
    let project_dir = project_dir.to_string();
    
    // Create a synchronous channel
    let (tx, rx) = mpsc::channel();
    
    // Spawn a standard thread to run the test in the background
    std::thread::spawn(move || {
        // Send a Running message right away
        let _ = tx.send(TestResult::Running);
        
        // Run the test synchronously (this is the blocking part)
        let result = run_jest_test(&test_file, &project_dir);
        
        // Send the completed result
        let _ = tx.send(TestResult::Completed(result));
    });
    
    rx
}