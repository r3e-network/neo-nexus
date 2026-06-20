use std::{fs, path::Path};

use anyhow::{Context, Result};

use super::markers::RequiredMarker;

pub(in crate::native_ui) fn cargo_dependency_present(
    root: &Path,
    dependency_name: &str,
) -> Result<bool> {
    let path = root.join("Cargo.toml");
    if !path.is_file() {
        return Ok(false);
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("failed to read Cargo manifest {}", path.display()))?;
    let manifest = toml::from_str::<toml::Value>(&text)
        .with_context(|| format!("failed to parse Cargo manifest {}", path.display()))?;
    let present = cargo_dependency_tables(&manifest).any(|table| {
        table.contains_key(dependency_name)
            || table.values().any(|value| {
                value
                    .as_table()
                    .and_then(|dependency| dependency.get("package"))
                    .and_then(toml::Value::as_str)
                    == Some(dependency_name)
            })
    });
    Ok(present)
}

fn cargo_dependency_tables(
    value: &toml::Value,
) -> impl Iterator<Item = &toml::map::Map<String, toml::Value>> {
    ["dependencies", "dev-dependencies", "build-dependencies"]
        .into_iter()
        .filter_map(|key| value.get(key).and_then(toml::Value::as_table))
}

pub(in crate::native_ui) fn cargo_dependency_requirements() -> [RequiredMarker; 2] {
    [
        RequiredMarker {
            path: "Cargo.toml",
            alternate_paths: &[],
            marker: "eframe",
            message: "Native desktop shell must use eframe instead of browser or WebView shells.",
        },
        RequiredMarker {
            path: "Cargo.toml",
            alternate_paths: &[],
            marker: "egui",
            message: "Native desktop shell must use egui widgets and panels.",
        },
    ]
}
