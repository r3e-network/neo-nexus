use anyhow::{Context, Result};
use url::Url;

pub(in crate::snapshots) fn safe_file_name(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("snapshot download file name is required");
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed == "." || trimmed == ".." {
        anyhow::bail!("snapshot download file name must not contain path separators");
    }
    Ok(trimmed.to_string())
}

pub(in crate::snapshots) fn file_name_from_url(value: &str) -> Result<String> {
    let url = Url::parse(value.trim()).context("snapshot download URL is invalid")?;
    let file_name = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|segment| !segment.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("snapshot download file name is required"))?;
    safe_file_name(file_name)
}
