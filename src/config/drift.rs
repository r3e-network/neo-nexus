//! What changed between a fresh render of a node's config and the file on disk.
//!
//! A node binary upgrade is where configs silently break: the file on disk was
//! written for a previous version's expectations, an operator hand-edited it,
//! or an older render predates the duty the workspace now records. Comparing
//! the file against what the workspace would render *right now* turns all
//! three into reviewable findings instead of a surprise at the next launch.
//!
//! The comparison is deliberately line-level and multiset-based. Config syntax
//! differs per client (neo-cli JSON, neo-go YAML, neo-rs/geth TOML), and a
//! syntax-aware merge would be a second opinion about meaning — which is the
//! validator's job, not this one.

use serde::Serialize;

/// How a file on disk differs from a fresh render of the same config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigLineDrift {
    /// Lines the file on disk has that a fresh render would not write: hand
    /// edits, or legacy keys a newer node version may no longer understand.
    pub unexpected_lines: usize,
    /// Lines a fresh render writes that the file on disk does not have.
    pub missing_lines: usize,
    /// Up to three of the unexpected lines, for the report an operator reads.
    pub unexpected_samples: Vec<String>,
}

impl ConfigLineDrift {
    /// Whether both texts carry the same lines.
    pub fn is_empty(&self) -> bool {
        self.unexpected_lines == 0 && self.missing_lines == 0
    }
}

/// How many unexpected lines a report quotes verbatim, and how wide each quote
/// may be: enough to recognise the edit, not enough to dump the file.
const SAMPLE_LINES: usize = 3;
const SAMPLE_MAX_CHARS: usize = 80;

/// Compares two config texts by line, as multisets: a line that appears more
/// often on disk than in the render counts once per extra occurrence, and the
/// reverse. Trailing newlines do not count; whitespace inside a line does.
pub fn line_drift(expected: &str, actual: &str) -> ConfigLineDrift {
    let mut counts: std::collections::HashMap<&str, i64> = std::collections::HashMap::new();
    for line in expected.lines() {
        *counts.entry(line).or_insert(0) -= 1;
    }
    for line in actual.lines() {
        *counts.entry(line).or_insert(0) += 1;
    }

    let mut drift = ConfigLineDrift {
        unexpected_lines: 0,
        missing_lines: 0,
        unexpected_samples: Vec::new(),
    };
    for surplus in counts.values() {
        if *surplus > 0 {
            drift.unexpected_lines += *surplus as usize;
        } else {
            drift.missing_lines += (-*surplus) as usize;
        }
    }

    // Samples come from the file in reading order, so a report is stable
    // across runs. A repeated line is quoted once, not once per occurrence —
    // the count above already says how many times it appears.
    let mut quoted: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for line in actual.lines() {
        if drift.unexpected_samples.len() >= SAMPLE_LINES {
            break;
        }
        if counts.get(line).copied().unwrap_or(0) > 0 && quoted.insert(line) {
            drift.unexpected_samples.push(shorten(line));
        }
    }

    drift
}

fn shorten(line: &str) -> String {
    let line = line.trim_end();
    if line.chars().count() <= SAMPLE_MAX_CHARS {
        line.to_string()
    } else {
        let cut: String = line.chars().take(SAMPLE_MAX_CHARS).collect();
        format!("{cut}…")
    }
}

#[cfg(test)]
#[path = "../../tests/unit/config/drift/tests.rs"]
mod tests;
