use std::{fs, path::Path, path::PathBuf};

use anyhow::{Context, Result};

use super::{
    model::SourceQualityFinding,
    rules::{blocked_markers, is_rust_source, is_test_source, should_skip_directory, snippet},
    MAX_RUST_SOURCE_LINES,
};

pub(super) struct SourceQualityScan {
    root: PathBuf,
    pub(super) scanned_files: usize,
    pub(super) scanned_directories: usize,
    pub(super) skipped_directories: Vec<String>,
    pub(super) findings: Vec<SourceQualityFinding>,
}

impl SourceQualityScan {
    pub(super) fn new(root: PathBuf) -> Self {
        Self {
            root,
            scanned_files: 0,
            scanned_directories: 0,
            skipped_directories: Vec::new(),
            findings: Vec::new(),
        }
    }

    pub(super) fn visit_dir(&mut self, directory: &Path) -> Result<()> {
        self.scanned_directories += 1;
        let mut entries = fs::read_dir(directory)
            .with_context(|| format!("failed to read source directory {}", directory.display()))?
            .collect::<std::io::Result<Vec<_>>>()
            .with_context(|| {
                format!("failed to inspect source directory {}", directory.display())
            })?;
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .with_context(|| format!("failed to inspect source path {}", path.display()))?;
            if file_type.is_symlink() {
                continue;
            }

            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if file_type.is_dir() {
                if should_skip_directory(&file_name) {
                    self.skipped_directories.push(self.relative_path(&path));
                    continue;
                }
                self.visit_dir(&path)?;
            } else if file_type.is_file() && is_rust_source(&path) {
                self.scanned_files += 1;
                self.scan_file(&path)?;
            }
        }

        Ok(())
    }

    fn scan_file(&mut self, path: &Path) -> Result<()> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read Rust source file {}", path.display()))?;
        let relative_path = self.relative_path(path);
        let line_count = text.lines().count();
        if line_count > MAX_RUST_SOURCE_LINES {
            self.findings.push(SourceQualityFinding {
                path: relative_path.clone(),
                line: MAX_RUST_SOURCE_LINES + 1,
                column: 1,
                marker: format!("{line_count} lines > {MAX_RUST_SOURCE_LINES}"),
                category: "oversized-rust-file".to_string(),
                snippet: "split this Rust source file into focused modules".to_string(),
            });
        }
        let is_test_source = is_test_source(path);
        for (line_index, line) in text.lines().enumerate() {
            self.scan_line(&relative_path, line_index + 1, line, is_test_source);
        }
        Ok(())
    }

    fn scan_line(&mut self, path: &str, line_number: usize, line: &str, is_test_source: bool) {
        for marker in blocked_markers() {
            if is_test_source && marker.is_test_assertion_shortcut() {
                continue;
            }
            let token = marker.token();
            let mut start = 0;
            while let Some(offset) = line[start..].find(&token) {
                let column = start + offset + 1;
                self.findings.push(SourceQualityFinding {
                    path: path.to_string(),
                    line: line_number,
                    column,
                    marker: token.clone(),
                    category: marker.category.to_string(),
                    snippet: snippet(line),
                });
                start += offset + token.len();
            }
        }
    }

    fn relative_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/")
    }
}
