use std::path::PathBuf;
use std::io;
use std::process::Command;

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