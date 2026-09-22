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
///
/// "Reachable" means a declaration the compiler would act on: a `mod x;` inside
/// a comment does not count, nor does one behind a statically false `#[cfg]`
/// (`any()`, `false`, `FALSE`, `not(all())` and combinations of those), nor
/// does a file whose own `#![cfg(...)]` compiles it away. Any other `cfg` —
/// `test`, a platform, a feature — is treated as mounted, because some build
/// compiles it. Limitation: a `mod x;` nested inside an inline `mod y { … }`
/// block is resolved as if it were at file level, so such a file is reported
/// as an orphan rather than silently accepted.
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
        "Detected {} orphan Rust source file(s) under {label} that no module tree or test target \
         compiles (unmounted, mounted only inside a comment, or behind a statically false cfg):\n{}",
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

    if reachable.contains(&canonical) {
        return Ok(());
    }

    let content = match fs::read_to_string(&canonical) {
        Ok(text) => text,
        Err(_) => return Ok(()),
    };

    let module = ModuleSource::parse(&content);
    if module.compiled_away {
        // `#![cfg(<false>)]`: the file is named by a `mod`, and compiles to
        // nothing. Leaving it unreached reports it, which is the point.
        return Ok(());
    }
    reachable.insert(canonical.clone());

    let parent_dir = canonical.parent().unwrap_or(&canonical);
    let file_name = canonical.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let stem = canonical.file_stem().and_then(|n| n.to_str()).unwrap_or("");

    let owns_stem_dir =
        !path_loaded && file_name != "lib.rs" && file_name != "main.rs" && file_name != "mod.rs";
    let sub_dir = if owns_stem_dir {
        parent_dir.join(stem)
    } else {
        parent_dir.to_path_buf()
    };

    for declaration in module.declarations {
        if let Some(custom) = declaration.custom_path {
            let target_path = parent_dir.join(custom);
            if target_path.exists() {
                // Reached through `#[path]`, so no stem-directory ownership.
                traverse_modules(&target_path, true, reachable)?;
            }
        } else {
            let candidate1 = sub_dir.join(format!("{}.rs", declaration.name));
            let candidate2 = sub_dir.join(&declaration.name).join("mod.rs");
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
    }

    Ok(())
}

/// What one source file tells the compiler about its out-of-line modules.
#[derive(Debug, Default)]
struct ModuleSource {
    /// `mod name;` declarations the compiler acts on, in source order.
    declarations: Vec<ModuleDeclaration>,
    /// The file opens with a statically false `#![cfg(...)]`.
    compiled_away: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct ModuleDeclaration {
    name: String,
    custom_path: Option<String>,
}

/// Outer attributes seen since the last item, waiting for the item they apply to.
#[derive(Debug, Default)]
struct PendingAttributes {
    custom_path: Option<String>,
    statically_disabled: bool,
}

impl ModuleSource {
    fn parse(source: &str) -> Self {
        let code = strip_comments(source);
        let mut module = Self::default();
        let mut pending = PendingAttributes::default();
        // An attribute whose closing `]` is on a later line.
        let mut open_attribute = String::new();
        // Inner attributes before the first item belong to the file; after it,
        // they can only open an inline `mod { … }` block.
        let mut seen_item = false;

        for line in code.lines() {
            let mut rest = if open_attribute.is_empty() {
                line.trim().to_string()
            } else {
                let joined = format!("{open_attribute} {}", line.trim());
                open_attribute.clear();
                joined
            };

            while rest.starts_with("#[") || rest.starts_with("#![") {
                let inner_attribute = rest.starts_with("#![");
                let open = if inner_attribute { 3 } else { 2 };
                let Some(close) = closing_bracket(&rest, open) else {
                    open_attribute = std::mem::take(&mut rest);
                    break;
                };
                let body = rest[open..close].trim();
                if inner_attribute {
                    if !seen_item
                        && cfg_predicate(body)
                            .is_some_and(|predicate| static_cfg(predicate) == Some(false))
                    {
                        module.compiled_away = true;
                    }
                } else if let Some(predicate) = cfg_predicate(body) {
                    if static_cfg(predicate) == Some(false) {
                        pending.statically_disabled = true;
                    }
                } else if let Some(custom_path) = path_attribute(body) {
                    pending.custom_path = Some(custom_path);
                }
                rest = rest[close + 1..].trim().to_string();
            }

            if rest.is_empty() {
                // Only attributes, or nothing: they apply to the next item.
                continue;
            }
            seen_item = true;
            if let Some(name) = parse_mod_declaration(&rest) {
                if !pending.statically_disabled {
                    module.declarations.push(ModuleDeclaration {
                        name: name.to_string(),
                        custom_path: pending.custom_path.take(),
                    });
                }
            }
            pending = PendingAttributes::default();
        }

        module
    }
}

/// `source` with every comment replaced by spaces and every newline kept, so
/// line structure survives. String and character literals are copied through
/// untouched, so a `//` or `/*` inside one does not start a comment. Block
/// comments nest, as they do in Rust.
fn strip_comments(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut code = String::with_capacity(source.len());
    let mut index = 0;

    while index < chars.len() {
        let current = chars[index];
        let next = chars.get(index + 1).copied();
        if current == '/' && next == Some('/') {
            while index < chars.len() && chars[index] != '\n' {
                code.push(' ');
                index += 1;
            }
        } else if current == '/' && next == Some('*') {
            let mut depth = 0usize;
            while index < chars.len() {
                let pair = (chars[index], chars.get(index + 1).copied());
                if pair == ('/', Some('*')) {
                    depth += 1;
                    code.push_str("  ");
                    index += 2;
                } else if pair == ('*', Some('/')) {
                    depth -= 1;
                    code.push_str("  ");
                    index += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    code.push(if chars[index] == '\n' { '\n' } else { ' ' });
                    index += 1;
                }
            }
        } else if current == '"' {
            index = copy_quoted(&chars, index, &mut code);
        } else if current == 'r' && raw_string_hashes(&chars, index).is_some() {
            index = copy_raw_string(&chars, index, &mut code);
        } else if current == '\'' {
            index = copy_char_literal(&chars, index, &mut code);
        } else {
            code.push(current);
            index += 1;
        }
    }

    code
}

/// Copy a `"…"` literal starting at `start`; returns the index just past it.
fn copy_quoted(chars: &[char], start: usize, code: &mut String) -> usize {
    code.push('"');
    let mut index = start + 1;
    while index < chars.len() {
        let current = chars[index];
        code.push(current);
        index += 1;
        if current == '\\' {
            if let Some(&escaped) = chars.get(index) {
                code.push(escaped);
                index += 1;
            }
        } else if current == '"' {
            break;
        }
    }
    index
}

/// The number of `#` in a raw string opening at `start` (`r"`, `r#"`, …).
fn raw_string_hashes(chars: &[char], start: usize) -> Option<usize> {
    let hashes = chars[start + 1..]
        .iter()
        .take_while(|&&character| character == '#')
        .count();
    (chars.get(start + 1 + hashes) == Some(&'"')).then_some(hashes)
}

fn copy_raw_string(chars: &[char], start: usize, code: &mut String) -> usize {
    let hashes = raw_string_hashes(chars, start).unwrap_or(0);
    let body_start = start + hashes + 2;
    for &character in &chars[start..body_start.min(chars.len())] {
        code.push(character);
    }
    let mut index = body_start;
    while index < chars.len() {
        let closes = chars[index] == '"'
            && chars[index + 1..]
                .iter()
                .take(hashes)
                .filter(|&&character| character == '#')
                .count()
                == hashes;
        if closes {
            for &character in &chars[index..(index + 1 + hashes).min(chars.len())] {
                code.push(character);
            }
            return index + 1 + hashes;
        }
        code.push(chars[index]);
        index += 1;
    }
    index
}

/// Copy a character literal (`'x'`, `'\n'`, `'"'`) starting at `start`, or just
/// the quote of a lifetime or label (`'a`), which has no closing quote.
fn copy_char_literal(chars: &[char], start: usize, code: &mut String) -> usize {
    code.push('\'');
    let mut index = start + 1;
    if chars.get(index) == Some(&'\\') {
        while index < chars.len() {
            let current = chars[index];
            code.push(current);
            index += 1;
            if current == '\\' {
                if let Some(&escaped) = chars.get(index) {
                    code.push(escaped);
                    index += 1;
                }
            } else if current == '\'' {
                break;
            }
        }
        return index;
    }
    if chars.get(index + 1) == Some(&'\'') {
        code.push(chars[index]);
        code.push('\'');
        return index + 2;
    }
    index
}

/// Index of the `]` closing an attribute whose body starts at `open`, or `None`
/// when the attribute continues on a later line.
fn closing_bracket(text: &str, open: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, character) in text.char_indices().skip_while(|&(index, _)| index < open) {
        if in_string {
            match character {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

/// The predicate of a `cfg(...)` attribute body; `cfg_attr` is not a gate.
fn cfg_predicate(body: &str) -> Option<&str> {
    let arguments = body.strip_prefix("cfg")?.trim_start().strip_prefix('(')?;
    arguments.trim_end().strip_suffix(')')
}

fn path_attribute(body: &str) -> Option<String> {
    let value = body
        .strip_prefix("path")?
        .trim_start()
        .strip_prefix('=')?
        .trim();
    let value = value.strip_prefix('"')?;
    let end = value.find('"')?;
    Some(value[..end].to_string())
}

/// Evaluate a cfg predicate as far as it can be known without a build
/// configuration: `Some(false)` only for predicates no configuration satisfies.
/// `FALSE` is the conventional never-set name (rustc's own test suite uses it).
fn static_cfg(predicate: &str) -> Option<bool> {
    let predicate = predicate.trim();
    if let Some(arguments) = cfg_call(predicate, "any") {
        let values: Vec<Option<bool>> = cfg_arguments(arguments).map(static_cfg).collect();
        if values.contains(&Some(true)) {
            return Some(true);
        }
        return values
            .iter()
            .all(|value| *value == Some(false))
            .then_some(false);
    }
    if let Some(arguments) = cfg_call(predicate, "all") {
        let values: Vec<Option<bool>> = cfg_arguments(arguments).map(static_cfg).collect();
        if values.contains(&Some(false)) {
            return Some(false);
        }
        return values
            .iter()
            .all(|value| *value == Some(true))
            .then_some(true);
    }
    if let Some(argument) = cfg_call(predicate, "not") {
        return static_cfg(argument).map(|value| !value);
    }
    match predicate {
        "false" | "FALSE" => Some(false),
        "true" => Some(true),
        _ => None,
    }
}

fn cfg_call<'a>(predicate: &'a str, name: &str) -> Option<&'a str> {
    let arguments = predicate
        .strip_prefix(name)?
        .trim_start()
        .strip_prefix('(')?;
    arguments.strip_suffix(')')
}

/// Split a cfg argument list on its top-level commas.
fn cfg_arguments(arguments: &str) -> impl Iterator<Item = &str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut start = 0;
    for (index, character) in arguments.char_indices() {
        match character {
            '"' => in_string = !in_string,
            '(' if !in_string => depth += 1,
            ')' if !in_string => depth = depth.saturating_sub(1),
            ',' if !in_string && depth == 0 => {
                parts.push(&arguments[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(&arguments[start..]);
    parts
        .into_iter()
        .map(str::trim)
        .filter(|part| !part.is_empty())
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

fn mounted(source: &str) -> Vec<String> {
    ModuleSource::parse(source)
        .declarations
        .into_iter()
        .map(|declaration| declaration.name)
        .collect()
}

#[test]
fn a_module_named_inside_a_block_comment_is_not_mounted() {
    assert_eq!(
        mounted("/*\nmod hidden;\n*/\nmod visible;\n"),
        vec!["visible"]
    );
    assert_eq!(mounted("/* mod one_line; */ mod after;\n"), vec!["after"]);
    // Block comments nest: the first `*/` does not end the outer comment.
    assert_eq!(
        mounted("/* outer /* inner */\nmod still_hidden;\n*/\nmod visible;\n"),
        vec!["visible"]
    );
}

#[test]
fn comment_markers_inside_literals_do_not_hide_modules() {
    let source = "const URL: &str = \"http://example.invalid/*\";\nconst QUOTE: char = '\"';\nconst RAW: &str = r#\"/* \"# ;\nmod after_literals; // trailing comment\n";
    assert_eq!(mounted(source), vec!["after_literals"]);
}

#[test]
fn a_module_behind_cfg_any_is_not_mounted() {
    assert_eq!(
        mounted("#[cfg(any())]\nmod gated;\n#[cfg(any())] mod gated_inline;\nmod kept;\n"),
        vec!["kept"]
    );
}

#[test]
fn a_module_behind_cfg_false_is_not_mounted() {
    assert_eq!(
        mounted("#[cfg(FALSE)]\nmod gated;\n#[cfg(false)]\n#[path = \"elsewhere.rs\"]\nmod gated_with_path;\n#[cfg(test)]\nmod tests;\n"),
        vec!["tests"]
    );
}

#[test]
fn a_statically_false_cfg_is_recognised_through_combinators() {
    assert_eq!(
        mounted("#[cfg(not(all()))]\nmod never;\n#[cfg(all(test, any()))]\nmod also_never;\n#[cfg(any(unix, windows))]\nmod on_some_platform;\n#[cfg(\n    any()\n)]\nmod never_multiline;\n"),
        vec!["on_some_platform"]
    );
}

#[test]
fn a_file_compiled_away_by_its_own_cfg_mounts_nothing() {
    let module = ModuleSource::parse("#![cfg(any())]\nmod child;\n");
    assert!(module.compiled_away);

    let module = ModuleSource::parse("#![cfg(test)]\nmod child;\n");
    assert!(!module.compiled_away);
    assert_eq!(
        module.declarations,
        vec![ModuleDeclaration {
            name: "child".to_string(),
            custom_path: None,
        }]
    );

    // After the first item an inner attribute can only open an inline module;
    // it compiles that block away, not the file.
    let module = ModuleSource::parse("mod child;\nmod inline {\n    #![cfg(any())]\n}\n");
    assert!(!module.compiled_away);
    assert_eq!(
        mounted("mod child;\nmod inline {\n    #![cfg(any())]\n}\n"),
        vec!["child"]
    );
}

#[test]
fn a_path_attribute_still_reaches_its_module() {
    assert_eq!(
        ModuleSource::parse("#[cfg(test)]\n#[path = \"../tests/unit/x/tests.rs\"]\nmod tests;\n")
            .declarations,
        vec![ModuleDeclaration {
            name: "tests".to_string(),
            custom_path: Some("../tests/unit/x/tests.rs".to_string()),
        }]
    );
}
