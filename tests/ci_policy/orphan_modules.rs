use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[test]
fn ci_policy_rejects_orphan_source_files_in_src() -> anyhow::Result<()> {
    let manifest_dir = manifest_root();
    let src_dir = manifest_dir.join("src");

    let mut all_sources = HashSet::new();
    collect_rust_files(&src_dir, &mut all_sources)?;

    let mut reachable_sources = HashSet::new();
    traverse_library_roots(&manifest_dir, &mut reachable_sources)?;

    assert_no_orphans(&manifest_dir, "src/", all_sources, reachable_sources)
}

/// A `.rs` file under `tests/` that no test target mounts is worse than a
/// missing test: the suite looks present on disk and in review, compiles into
/// nothing, and the gate stays green while its assertions never run. That is
/// how `tests/integration/` sat with 68 `#[test]` functions behind a one-test
/// print stub. Every file under `tests/` must be reachable from a real crate
/// root — either a `tests/*.rs` test target or the library/binary that mounts
/// unit-test files via `#[path]`.
#[test]
fn ci_policy_rejects_unmounted_test_modules() -> anyhow::Result<()> {
    let manifest_dir = manifest_root();
    let tests_dir = manifest_dir.join("tests");

    let mut all_sources = HashSet::new();
    collect_rust_files(&tests_dir, &mut all_sources)?;

    let mut reachable_sources = HashSet::new();
    // Unit-test files live under tests/unit/ but are mounted from src/ via
    // `#[path = "../../tests/unit/…"]`, so the library tree can reach them.
    traverse_library_roots(&manifest_dir, &mut reachable_sources)?;
    for root in test_target_roots(&manifest_dir)? {
        traverse_modules(&root, false, &mut reachable_sources)?;
    }

    assert_no_orphans(&manifest_dir, "tests/", all_sources, reachable_sources)
}

fn manifest_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::canonicalize(manifest_dir).unwrap_or_else(|_| manifest_dir.to_path_buf())
}

fn traverse_library_roots(
    manifest_dir: &Path,
    reachable_sources: &mut HashSet<PathBuf>,
) -> anyhow::Result<()> {
    let src_dir = manifest_dir.join("src");
    let lib_rs = src_dir.join("lib.rs");
    let main_rs = src_dir.join("main.rs");

    if lib_rs.exists() {
        traverse_modules(&lib_rs, false, reachable_sources)?;
    }
    if main_rs.exists() {
        traverse_modules(&main_rs, false, reachable_sources)?;
    }
    Ok(())
}

/// Every `tests/*.rs` file is its own integration-test crate root.
fn test_target_roots(manifest_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let tests_dir = manifest_dir.join("tests");
    let mut roots = Vec::new();
    for entry in fs::read_dir(&tests_dir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            roots.push(path);
        }
    }
    roots.sort();
    Ok(roots)
}

fn assert_no_orphans(
    manifest_dir: &Path,
    label: &str,
    all_sources: HashSet<PathBuf>,
    reachable_sources: HashSet<PathBuf>,
) -> anyhow::Result<()> {
    let mut orphan_files = Vec::new();
    for source in &all_sources {
        if !reachable_sources.contains(source) {
            let relative = source
                .strip_prefix(manifest_dir)
                .unwrap_or(source)
                .to_string_lossy()
                .replace('\\', "/");
            orphan_files.push(relative);
        }
    }

    orphan_files.sort();

    assert!(
        orphan_files.is_empty(),
        "Detected {} orphan Rust source file(s) under {label} that are not reachable from any module tree or test target:\n{}",
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

/// Walk `file_path`'s `mod` tree, recording every file reached.
///
/// `path_loaded` must be true when `file_path` was itself reached through a
/// `#[path = …]` attribute. Rust then gives that module no stem-directory
/// ownership: its `mod foo;` children sit beside it (`…/foo.rs`) rather than
/// under `…/<stem>/foo.rs`. `tests/unit/cli/tests.rs` is the live example —
/// `mod reports;` finds `tests/unit/cli/reports.rs`, and only the next level
/// down (`reports.rs` itself, loaded by plain `mod`) goes back to the stem rule
/// and finds `tests/unit/cli/reports/event_journal.rs`. Guessing one rule for
/// both shapes reports half of tests/unit as orphans and trains authors to
/// ignore the gate.
fn traverse_modules(
    file_path: &Path,
    path_loaded: bool,
    reachable: &mut HashSet<PathBuf>,
) -> anyhow::Result<()> {
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

    let owns_stem_dir = !path_loaded
        && file_name != "lib.rs"
        && file_name != "main.rs"
        && file_name != "mod.rs";
    let sub_dir = if owns_stem_dir {
        parent_dir.join(stem)
    } else {
        parent_dir.to_path_buf()
    };

    let mut pending_custom_path: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(path_str) = parse_path_attribute(trimmed) {
            pending_custom_path = Some(path_str);
            continue;
        }

        if let Some(mod_name) = parse_mod_declaration(trimmed) {
            if let Some(custom) = pending_custom_path.take() {
                let target_path = parent_dir.join(custom);
                if target_path.exists() {
                    // Reached through `#[path]`, so no stem-directory ownership.
                    traverse_modules(&target_path, true, reachable)?;
                }
            } else {
                let candidate1 = sub_dir.join(format!("{mod_name}.rs"));
                let candidate2 = sub_dir.join(mod_name).join("mod.rs");
                let target_path = if candidate1.exists() {
                    candidate1
                } else if candidate2.exists() {
                    candidate2
                } else {
                    candidate1
                };

                if target_path.exists() {
                    traverse_modules(&target_path, false, reachable)?;
                }
            }
        } else if !trimmed.starts_with('#') {
            pending_custom_path = None;
        }
    }

    Ok(())
}

/// Remove a leading visibility, whatever its scope.
///
/// This handled `pub`, `pub(crate)` and `pub(super)` by name and nothing else,
/// so a module declared `pub(in crate::some::path) mod x;` — valid Rust, and the
/// tightest visibility that works for a helper shared between two sibling
/// modules — was invisible to the traversal and its file was reported as an
/// orphan. A gate that cannot read the language it guards pushes authors toward
/// wider visibility than they need.
fn strip_visibility(line: &str) -> &str {
    let Some(rest) = line.strip_prefix("pub") else {
        return line;
    };
    match rest.strip_prefix('(') {
        // `pub(...)`: skip to the matching parenthesis. Visibility scopes do not
        // nest, so the first `)` closes it.
        Some(scoped) => scoped
            .find(')')
            .map_or(line, |end| scoped[end + 1..].trim_start()),
        // `pub mod`, and not `public_thing`.
        None if rest.starts_with(char::is_whitespace) => rest.trim_start(),
        None => line,
    }
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
    let without_pub = strip_visibility(line);

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
