use std::path::PathBuf;
use std::io;
use std::fs;
use regex::Regex;

/// Finds a Jest configuration file in the given directory.
pub fn find_jest_config_file(dir: &PathBuf) -> io::Result<Option<PathBuf>> {
    // List of possible Jest config filenames
    let config_filenames = [
        "jest.config.js",
        "jest.config.ts",
        "jest.config.mjs",
        "jest.config.cjs",
        "jest.config.json",
        ".jestrc",
        ".jestrc.js",
        ".jestrc.json",
    ];
    
    // Also check in package.json (common for Jest config)
    let package_json = dir.join("package.json");
    if package_json.exists() {
        if let Ok(content) = fs::read_to_string(&package_json) {
            if content.contains("\"jest\"") {
                println!("Found Jest configuration in package.json");
                return Ok(Some(package_json));
            }
        }
    }
    
    // Look for dedicated Jest config files
    for filename in config_filenames.iter() {
        let config_path = dir.join(filename);
        if config_path.exists() {
            println!("Found Jest configuration at {}", config_path.display());
            return Ok(Some(config_path));
        }
    }
    
    // Look for Jest config in parent directory (up to 3 levels)
    let mut parent_dir = dir.parent().map(PathBuf::from);
    let mut level = 0;
    while let Some(parent) = parent_dir {
        if level >= 3 {
            break; // Don't go too far up
        }
        
        for filename in config_filenames.iter() {
            let config_path = parent.join(filename);
            if config_path.exists() {
                println!("Found Jest configuration at {}", config_path.display());
                return Ok(Some(config_path));
            }
        }
        
        parent_dir = parent.parent().map(PathBuf::from);
        level += 1;
    }
    
    println!("No Jest configuration file found");
    Ok(None)
}

/// Extracts testMatch patterns from a Jest configuration file.
pub fn extract_test_matches(config_path: &PathBuf) -> io::Result<Vec<String>> {
    let content = fs::read_to_string(config_path)?;
    
    // Extract testMatch array using regex
    // This is a simple extraction - a real implementation might use a JS parser
    let test_match_regex = Regex::new(r#"testMatch\s*:?\s*\[\s*(["'][^"']+["'](?:\s*,\s*["'][^"']+["'])*)\s*\]"#)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    if let Some(captures) = test_match_regex.captures(&content) {
        if let Some(patterns_match) = captures.get(1) {
            let patterns_str = patterns_match.as_str();
            
            // Split by comma and extract the patterns
            let patterns: Vec<String> = patterns_str
                .split(',')
                .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
                .collect();
            
            println!("Found testMatch patterns: {:?}", patterns);
            return Ok(patterns);
        }
    }
    
    // Alternative pattern - check for testMatch: [values]
    // This is for different formatting styles
    let alt_regex = Regex::new(r#"["']testMatch["']\s*:?\s*\[\s*(["'][^"']+["'](?:\s*,\s*["'][^"']+["'])*)\s*\]"#)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    if let Some(captures) = alt_regex.captures(&content) {
        if let Some(patterns_match) = captures.get(1) {
            let patterns_str = patterns_match.as_str();
            
            // Split by comma and extract the patterns
            let patterns: Vec<String> = patterns_str
                .split(',')
                .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
                .collect();
            
            println!("Found testMatch patterns (alt format): {:?}", patterns);
            return Ok(patterns);
        }
    }
    
    // If all else fails, return default patterns
    Ok(vec![
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
    ])
}

/// Finds test files matching the given patterns in the specified directory.
pub fn find_matching_tests(
    test_matches: &[String],
    project_root: &PathBuf,
) -> io::Result<Vec<String>> {
    use glob::glob;
    
    let mut results = Vec::new();
    let canonical_root = project_root.canonicalize()?;

    // Loop through each glob pattern (e.g. "**/*.test.ts")
    for pattern in test_matches {
        // Join the pattern with the project path
        let full_pattern = project_root.join(pattern);

        // Convert the full path to a string (for glob)
        let pattern_str = full_pattern.to_string_lossy().to_string();

        // Try to run the glob — continue to next pattern if invalid
        let paths = match glob(&pattern_str) {
            Ok(paths) => paths,
            Err(err) => {
                eprintln!("Invalid pattern '{}': {}", pattern, err);
                continue;
            }
        };

        // Add each matching path to our results
        for path_result in paths {
            if let Ok(path) = path_result {
                // Skip files in node_modules directories
                if path.to_string_lossy().contains("/node_modules/") {
                    continue;
                }
                
                // Try to make the path relative to the search directory
                let display_path = if let Ok(rel_path) = path.strip_prefix(&canonical_root) {
                    rel_path.display().to_string()
                } else {
                    path.display().to_string()
                };
                
                results.push(display_path);
            }
        }
    }

    // Sort results alphabetically for better readability
    results.sort();
    
    Ok(results)
}