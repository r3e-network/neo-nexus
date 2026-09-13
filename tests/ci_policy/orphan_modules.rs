use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[test]
fn ci_policy_rejects_orphan_source_files_in_src() -> anyhow::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest_dir =
        fs::canonicalize(manifest_dir).unwrap_or_else(|_| manifest_dir.to_path_buf());
    let src_dir = manifest_dir.join("src");

    let mut all_sources = HashSet::new();
    collect_rust_files(&src_dir, &mut all_sources)?;

    let mut reachable_sources = HashSet::new();
    let lib_rs = src_dir.join("lib.rs");
    let main_rs = src_dir.join("main.rs");

    if lib_rs.exists() {
        traverse_modules(&lib_rs, &mut reachable_sources)?;
    }
    if main_rs.exists() {
        traverse_modules(&main_rs, &mut reachable_sources)?;
    }

    let mut orphan_files = Vec::new();
    for source in &all_sources {
        if !reachable_sources.contains(source) {
            let relative = source
                .strip_prefix(&manifest_dir)
                .unwrap_or(source)
                .to_string_lossy()
                .replace('\\', "/");
            orphan_files.push(relative);
        }
    }

    orphan_files.sort();

    assert!(
        orphan_files.is_empty(),
        "Detected {} orphan Rust source file(s) under src/ that are not reachable from module tree:\n{}",
        orphan_files.len(),
        orphan_files
            .iter()
            .map(|path| format!("  - {path}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(())
}

fn collect_rust_files(dir: &Path, files: &mut HashSet<PathBuf>) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            let canonical = fs::canonicalize(&path)?;
            files.insert(canonical);
        }
    }
    Ok(())
}

fn traverse_modules(file_path: &Path, reachable: &mut HashSet<PathBuf>) -> anyhow::Result<()> {
    let canonical = match fs::canonicalize(file_path) {
        Ok(path) => path,
        Err(_) => return Ok(()),
    };

    if !reachable.insert(canonical.clone()) {
        return Ok(());
    }

    let content = match fs::read_to_string(&canonical) {
        Ok(text) => text,
        Err(_) => return Ok(()),
    };

    let parent_dir = canonical.parent().unwrap_or(&canonical);
    let file_name = canonical.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let stem = canonical.file_stem().and_then(|n| n.to_str()).unwrap_or("");

    let sub_dir = if file_name == "lib.rs" || file_name == "main.rs" || file_name == "mod.rs" {
        parent_dir.to_path_buf()
    } else {
        parent_dir.join(stem)
    };

    let mut pending_custom_path: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(path_str) = parse_path_attribute(trimmed) {
            pending_custom_path = Some(path_str);
            continue;
        }

        if let Some(mod_name) = parse_mod_declaration(trimmed) {
            let target_path = if let Some(custom) = pending_custom_path.take() {
                parent_dir.join(custom)
            } else {
                let candidate1 = sub_dir.join(format!("{mod_name}.rs"));
                let candidate2 = sub_dir.join(mod_name).join("mod.rs");
                if candidate1.exists() {
                    candidate1
                } else if candidate2.exists() {
                    candidate2
                } else {
                    candidate1
                }
            };

            if target_path.exists() {
                traverse_modules(&target_path, reachable)?;
            }
        } else if !trimmed.starts_with('#') {
            pending_custom_path = None;
        }
    }

    Ok(())
}

fn parse_path_attribute(line: &str) -> Option<String> {
    if !line.starts_with("#[path") {
        return None;
    }
    let quote_start = line.find('"')?;
    let quote_end = line[quote_start + 1..].find('"')? + quote_start + 1;
    Some(line[quote_start + 1..quote_end].to_string())
}

fn parse_mod_declaration(line: &str) -> Option<&str> {
    let without_pub = if let Some(stripped) = line.strip_prefix("pub ") {
        stripped.trim_start()
    } else if let Some(stripped) = line.strip_prefix("pub(crate) ") {
        stripped.trim_start()
    } else if let Some(stripped) = line.strip_prefix("pub(super) ") {
        stripped.trim_start()
    } else {
        line
    };

    let without_mod = without_pub.strip_prefix("mod ")?;
    let trimmed = without_mod.trim_start();
    let mod_name = trimmed.strip_suffix(';')?;
    let mod_name = mod_name.trim();

    if mod_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !mod_name.is_empty()
    {
        Some(mod_name)
    } else {
        None
    }
}
