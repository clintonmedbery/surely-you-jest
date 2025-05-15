use color_eyre::Result;
use std::{env, path::PathBuf};

mod app;
mod jest;
mod ui;
mod widgets;

use app::App;
use jest::config_finder;

fn main() -> Result<()> {
    color_eyre::install()?;

    // Get path to directory containing tests from CLI args
    let path = env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        eprintln!("Usage: cargo run -- <path-to-tests-directory>");
        std::process::exit(1);
    });

    if !path.exists() || !path.is_dir() {
        eprintln!("The specified path does not exist or is not a directory: {}", path.display());
        std::process::exit(1);
    }

    // Try to find and read Jest config file
    let test_matches = match config_finder::find_jest_config_file(&path)? {
        Some(config_path) => {
            println!("Using Jest configuration from {}", config_path.display());
            config_finder::extract_test_matches(&config_path)?
        },
        None => {
            println!("Using default test patterns");
            // Fallback to default patterns if no config found
            vec![
                "**/*.test.js".to_string(),
                "**/*.test.ts".to_string(),
                "**/*.test.tsx".to_string(),
                "**/*.test.jsx".to_string(),
                "**/*.spec.js".to_string(),
                "**/*.spec.ts".to_string(),
                "**/*.spec.tsx".to_string(),
                "**/*.spec.jsx".to_string(),
                "**/__tests__/**/*.js".to_string(),
                "**/__tests__/**/*.ts".to_string(),
            ]
        }
    };
    
    let tests = config_finder::find_matching_tests(&test_matches, &path)?;
    let path_str = path.display().to_string();

    // Initialize the terminal
    let terminal = ratatui::init();
    
    // Create and run the application
    let result = App::new(path_str, test_matches, tests).run(terminal);
    
    // Restore terminal state
    ratatui::restore();
    
    // Return the result
    result
}