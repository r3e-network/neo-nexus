use std::{fs, path::Path};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub(super) struct AuditFile {
    pub(super) relative_path: String,
    pub(super) text: String,
}

pub(super) fn load_audit_files(root: &Path) -> Result<Vec<AuditFile>> {
    let mut files = Vec::new();
    push_file_if_present(root, Path::new("Cargo.toml"), &mut files)?;
    push_file_if_present(root, Path::new("src/main.rs"), &mut files)?;
    push_file_if_present(root, Path::new("src/app.rs"), &mut files)?;
    let app_dir = root.join("src").join("app");
    if app_dir.is_dir() {
        visit_app_dir(root, &app_dir, &mut files)?;
    }
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

fn visit_app_dir(root: &Path, directory: &Path, files: &mut Vec<AuditFile>) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("failed to read UI directory {}", directory.display()))?
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("failed to inspect UI directory {}", directory.display()))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect UI path {}", path.display()))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            visit_app_dir(root, &path, files)?;
        } else if file_type.is_file() && is_rust_file(&path) {
            push_file(root, &path, files)?;
        }
    }
    Ok(())
}

fn push_file_if_present(
    root: &Path,
    relative_path: &Path,
    files: &mut Vec<AuditFile>,
) -> Result<()> {
    let path = root.join(relative_path);
    if path.is_file() {
        push_file(root, &path, files)?;
    }
    Ok(())
}

fn push_file(root: &Path, path: &Path, files: &mut Vec<AuditFile>) -> Result<()> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read native UI source {}", path.display()))?;
    files.push(AuditFile {
        relative_path: relative_path(root, path),
        text,
    });
    Ok(())
}

fn is_rust_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}
