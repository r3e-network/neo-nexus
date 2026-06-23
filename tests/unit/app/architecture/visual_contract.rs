//! Visual-contract assertions split out of the main architecture file to keep
//! it under the source-quality module size limit. These checks scan view source
//! for rendering-contract violations that a screenshot cannot enforce uniformly.

use std::path::{Path, PathBuf};

fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn rust_sources_under(root_dir: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let mut pending_dirs = vec![manifest_path(root_dir)];
    while let Some(dir) = pending_dirs.pop() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                pending_dirs.push(path);
            } else if path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs") {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

/// Error states must render with a semantic danger/warning colour, never as
/// plain or muted text. A workbench that surfaces some failures in red and
/// others in grey reads as inconsistent and understates real problems. Every
/// view source line that turns an error into displayed text must apply a colour.
#[test]
fn error_states_render_with_a_semantic_colour() -> anyhow::Result<()> {
    for path in rust_sources_under("src/app/views")? {
        let source = std::fs::read_to_string(&path)?;
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            // Lines that display an error's message string.
            let shows_error =
                trimmed.contains("error.to_string()") || trimmed.contains("err.to_string()");
            if !shows_error {
                continue;
            }
            // `empty_state` deliberately renders a guidance-style message (title
            // + muted body), not an inline error, so it is exempt.
            if trimmed.contains("empty_state(") {
                continue;
            }
            // Such a line must carry an explicit colour, either inline or on the
            // immediately following label/RichText continuation. We accept the
            // semantic danger/warning helpers, the status-color mapping, and the
            // dedicated fact_error widget (which colours its value internally).
            let window = source
                .lines()
                .skip(index)
                .take(3)
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                window.contains(".color(") || window.contains("fact_error("),
                "{}:{} renders an error without a semantic colour; use theme::danger()/warning():\n{trimmed}",
                path.display(),
                index + 1,
            );
        }
    }
    Ok(())
}
