use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use anyhow::{Context, Result};

use super::{
    diagnosis::diagnose_snapshot,
    model::{LogDiagnosis, LogLine, LogSnapshot},
};

pub struct LogReader;

impl LogReader {
    pub fn snapshot(path: impl AsRef<Path>, max_bytes: usize) -> Result<LogSnapshot> {
        let path = path.as_ref().to_path_buf();
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(LogSnapshot {
                    path,
                    exists: false,
                    bytes: 0,
                    truncated: false,
                    lines: Vec::new(),
                });
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to open log {}", path.display()));
            }
        };

        let bytes = file
            .metadata()
            .with_context(|| format!("failed to inspect log {}", path.display()))?
            .len();
        let max_bytes = max_bytes.max(1) as u64;
        let start = bytes.saturating_sub(max_bytes);
        file.seek(SeekFrom::Start(start))
            .with_context(|| format!("failed to seek log {}", path.display()))?;

        let mut buffer = Vec::new();
        file.take(max_bytes)
            .read_to_end(&mut buffer)
            .with_context(|| format!("failed to read log {}", path.display()))?;

        let text = String::from_utf8_lossy(&buffer);
        let lines = if start > 0 {
            text.find('\n')
                .map_or_else(Vec::new, |index| collect_lines(&text[index + 1..]))
        } else {
            collect_lines(&text)
        };

        Ok(LogSnapshot {
            path,
            exists: true,
            bytes,
            truncated: start > 0,
            lines,
        })
    }

    pub fn clear(path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create log directory {}", parent.display()))?;
        }

        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .with_context(|| format!("failed to clear log {}", path.display()))?;
        Ok(())
    }

    pub fn filtered_lines(snapshot: &LogSnapshot, query: &str) -> Vec<LogLine> {
        let query = query.trim().to_ascii_lowercase();
        snapshot
            .lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| {
                if query.is_empty() || line.to_ascii_lowercase().contains(&query) {
                    Some(LogLine {
                        number: index + 1,
                        text: line.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn diagnose(snapshot: &LogSnapshot) -> LogDiagnosis {
        diagnose_snapshot(snapshot)
    }
}

fn collect_lines(text: &str) -> Vec<String> {
    text.lines().map(ToString::to_string).collect()
}
