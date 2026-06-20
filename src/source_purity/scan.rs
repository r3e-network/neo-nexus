use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use super::{
    cargo::cargo_dependency_findings,
    rules::{disallowed_directory, disallowed_file, should_skip_directory},
    SourcePurityFinding,
};

pub(super) struct SourcePurityScan {
    root: PathBuf,
    pub(super) scanned_files: usize,
    pub(super) scanned_directories: usize,
    pub(super) skipped_directories: Vec<String>,
    pub(super) findings: Vec<SourcePurityFinding>,
}

impl SourcePurityScan {
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
                if let Some(finding) = disallowed_directory(&file_name, self.relative_path(&path)) {
                    self.findings.push(finding);
                    continue;
                }
                self.visit_dir(&path)?;
            } else if file_type.is_file() {
                self.scanned_files += 1;
                let relative_path = self.relative_path(&path);
                if let Some(finding) = disallowed_file(&file_name, relative_path.clone()) {
                    self.findings.push(finding);
                }
                self.findings.extend(cargo_dependency_findings(
                    &file_name,
                    &relative_path,
                    &path,
                )?);
            }
        }

        Ok(())
    }

    fn relative_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/")
    }
}
