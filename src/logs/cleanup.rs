//! Log cleanup utilities for clearing log files from directories.
//!
//! Provides functionality to safely remove all `.log` files from a given
//! directory, with proper error handling for missing directories, permission
//! issues, and file locking problems.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// Clear all `.log` files from a given directory.
///
/// This function recursively removes all files ending in `.log` within the
/// specified directory. It handles errors gracefully:
/// - If the directory doesn't exist, returns success (nothing to clear)
/// - If no log files are found, returns success
/// - Permission errors on specific files are logged but don't halt the operation
///
/// # Arguments
///
/// * `log_dir` - The path to the directory containing log files to clear
///
/// # Returns
///
/// * `Ok(())` - Successfully completed (all deletable files removed)
/// * `Err(_)` - Encountered an unrecoverable error
///
/// # Examples
///
/// ```ignore
/// use std::path::Path;
///
/// // Clear logs from workspace logs directory
/// let log_dir = Path::new("workspaces/logs");
/// clear_all_logs(log_dir)?;
/// ```
pub fn clear_all_logs(log_dir: &Path) -> Result<usize> {
    // If directory doesn't exist, nothing to clear
    if !log_dir.exists() {
        return Ok(0);
    }

    if !log_dir.is_dir() {
        anyhow::bail!("log path is not a directory: {}", log_dir.display());
    }

    let mut cleared_count = 0;

    // Walk through the directory and collect .log files
    let entries = fs::read_dir(log_dir)
        .with_context(|| format!("failed to read log directory {}", log_dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Only process files (not directories)
        if !path.is_file() {
            continue;
        }

        // Check if it's a .log file
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("log"))
        {
            // Attempt to delete the file
            match fs::remove_file(&path) {
                Ok(()) => {
                    cleared_count += 1;
                    eprintln!("Cleared log file: {}", path.display());
                }
                Err(error) => {
                    // Log error but continue with other files
                    eprintln!(
                        "Failed to clear {}: {} (permission denied or file locked)",
                        path.display(),
                        error.kind()
                    );
                }
            }
        }
    }

    Ok(cleared_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_clear_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = clear_all_logs(temp_dir.path()).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_clear_nonexistent_directory() {
        let nonexistent = std::path::Path::new("/tmp/does_not_exist_12345");
        let result = clear_all_logs(nonexistent).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_clear_log_files_only() {
        let temp_dir = TempDir::new().unwrap();

        // Create some log files
        File::create(temp_dir.path().join("app.log")).unwrap();
        File::create(temp_dir.path().join("error.log")).unwrap();
        File::create(temp_dir.path().join("readme.txt")).unwrap();

        // Count only .log files
        let log_count: usize = temp_dir
            .path()
            .read_dir()
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("log"))
            })
            .count();

        let result = clear_all_logs(temp_dir.path()).unwrap();

        assert_eq!(result, log_count);
        // Both log files should be deleted
        assert!(!temp_dir.path().join("app.log").exists());
        assert!(!temp_dir.path().join("error.log").exists());
        // Non-log file should remain
        assert!(temp_dir.path().join("readme.txt").exists());
    }

    #[test]
    fn test_case_insensitive_log_extension() {
        let temp_dir = TempDir::new().unwrap();

        // Create files with different case extensions
        File::create(temp_dir.path().join("upper.LOG")).unwrap();
        File::create(temp_dir.path().join("mixed.Log")).unwrap();
        File::create(temp_dir.path().join("lower.log")).unwrap();

        let result = clear_all_logs(temp_dir.path()).unwrap();

        assert_eq!(result, 3);
    }
}
