//! Filesystem operations module.
//!
//! Provides file search and directory counting functionality:
//! - Find files by exact name, glob pattern, or partial match
//! - Count files and directories in a path

use std::fs;
use std::path::Path;
use stool_core::error::{Result, StoolError, StoolErrorType};
/// Finds files matching the given pattern.
///
/// Supports three pattern types:
/// - Exact match: `"file.txt"` (files with extension)
/// - Glob pattern: `"*.rs"` (contains `*` or `?`)
/// - Partial match: `"partial"` (wrapped as `*partial*`)
///
/// Searches recursively, skipping hidden directories.
///
/// # Arguments
/// * `pattern` - Search pattern (exact name, glob, or partial match)
/// * `path` - Optional search directory (defaults to current directory)
///
/// # Errors
/// Returns error if path doesn't exist or pattern is invalid
pub fn find(pattern: &str, path: Option<&str>) -> Result<()> {
    let search_path = path.unwrap_or(".");
    let search_dir = Path::new(search_path);

    if !search_dir.exists() {
        return Err(StoolError::new(StoolErrorType::FileNotFound)
            .with_message(format!("Path not found: {}", search_path)));
    }

    // Determine pattern type
    let (is_exact, search_pattern) = if pattern.contains('*') || pattern.contains('?') {
        // Glob pattern
        (false, pattern.to_string())
    } else if pattern.contains('.') && !pattern.starts_with('.') {
        // Exact filename (has extension)
        (true, pattern.to_string())
    } else {
        // Partial match
        (false, format!("*{}*", pattern))
    };

    println!("Searching for '{}' in {}...", pattern, search_path);

    let mut results = Vec::new();
    search_recursive(search_dir, &search_pattern, is_exact, &mut results)?;

    if results.is_empty() {
        println!("No files found matching '{}'", pattern);
    } else {
        println!("\nFound {} file(s):", results.len());
        for result in results {
            println!("  {}", result);
        }
    }

    Ok(())
}

/// Counts files and directories in the given path.
///
/// Performs a non-recursive count of immediate children in the directory.
///
/// # Arguments
/// * `path` - Optional target directory (defaults to current directory)
///
/// # Errors
/// Returns error if path doesn't exist or is not a directory
pub fn count(path: Option<&str>) -> Result<()> {
    let target_path = path.unwrap_or(".");
    let dir = Path::new(target_path);

    if !dir.exists() {
        return Err(StoolError::new(StoolErrorType::FileNotFound)
            .with_message(format!("Path not found: {}", target_path)));
    }

    if !dir.is_dir() {
        return Err(StoolError::new(StoolErrorType::InvalidInput)
            .with_message(format!("Not a directory: {}", target_path)));
    }

    let entries = fs::read_dir(dir).map_err(|e| {
        StoolError::new(StoolErrorType::IoError)
            .with_message(format!("Failed to read directory: {}", target_path))
            .with_source(e)
    })?;

    let count = entries.count();
    println!("{} items in {}", count, target_path);

    Ok(())
}

// Recursive directory search
fn search_recursive(
    dir: &Path,
    pattern: &str,
    is_exact: bool,
    results: &mut Vec<String>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|e| {
        StoolError::new(StoolErrorType::IoError)
            .with_message(format!("Failed to read directory: {}", dir.display()))
            .with_source(e)
    })?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Skip hidden directories (except current search root)
        if path.is_dir()
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        {
            continue;
        }

        // Check file name against pattern
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            let matches = if is_exact {
                filename == pattern
            } else {
                matches_glob(filename, pattern)?
            };

            if matches {
                results.push(path.display().to_string());
            }
        }

        // Recurse into subdirectories
        if path.is_dir() {
            search_recursive(&path, pattern, is_exact, results)?;
        }
    }

    Ok(())
}

// Simple glob matching (* and ?)
fn matches_glob(text: &str, pattern: &str) -> Result<bool> {
    let re_pattern = pattern
        .replace(".", "\\.")
        .replace("*", ".*")
        .replace("?", ".");

    let regex = regex::Regex::new(&format!("^{}$", re_pattern)).map_err(|e| {
        StoolError::new(StoolErrorType::SearchPatternInvalid)
            .with_message(format!("Invalid pattern: {}", pattern))
            .with_source(e)
    })?;

    Ok(regex.is_match(text))
}
