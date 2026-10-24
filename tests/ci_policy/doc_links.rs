//! A link in a document is navigation, and one that points at nothing is worse
//! than no link: it looks like an answer and costs a click to learn it is not.
//!
//! Three ways this rots, all of which happened in this repository:
//!
//! 1. A module that was a file became a directory — `src/web/html.rs` is now
//!    `src/web/html/`, and the link kept pointing at the file.
//! 2. A section moved to another document — the private-network guide lives in
//!    `native-rust.md`, not in `web.md`.
//! 3. A link written from inside `docs/` as if it were the repository root —
//!    `docs/web.md` resolves to `docs/docs/web.md` and nothing opens.
//!
//! None of them failed anything, and all of them shipped.
//!
//! Only the file part of a link is checked. An anchor that no longer exists
//! inside a file that still does is not detected, because the slug rules differ
//! between the renderers this documentation is read in — a false alarm there
//! would get this test switched off, which is the one outcome that fixes
//! nothing.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

fn repository_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::canonicalize(manifest_dir).unwrap_or_else(|_| manifest_dir.to_path_buf())
}

/// Every Markdown file a reader can reach from the repository: the top level
/// and `docs/`. Deliberately not recursive — the point is the documents people
/// land on, not every file a crate dependency happens to ship.
fn documents(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut documents = Vec::new();
    for directory in [root.to_path_buf(), root.join("docs")] {
        for entry in fs::read_dir(&directory)? {
            let path = entry?.path();
            if path.is_file() && path.extension().is_some_and(|extension| extension == "md") {
                documents.push(path);
            }
        }
    }
    documents.sort();
    Ok(documents)
}

/// The link targets in one document, as written. Skips absolute URLs, in-page
/// anchors and protocol-relative links, none of which name a file here.
fn link_targets(markdown: &str) -> BTreeSet<String> {
    let mut targets = BTreeSet::new();
    let mut rest = markdown;
    while let Some(open) = rest.find("](") {
        rest = &rest[open + 2..];
        let Some(close) = rest.find(')') else {
            break;
        };
        let target = &rest[..close];
        rest = &rest[close..];
        // `[text](path "title")` — the title is not part of the path.
        let target = target.split_whitespace().next().unwrap_or("");
        if target.is_empty()
            || target.starts_with('#')
            || target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("mailto:")
            || target.starts_with("//")
        {
            continue;
        }
        // The anchor names a place inside the target, not a second file.
        let path = target.split('#').next().unwrap_or("");
        if !path.is_empty() {
            targets.insert(path.to_string());
        }
    }
    targets
}

#[test]
fn ci_policy_rejects_a_link_to_a_file_that_is_not_there() -> anyhow::Result<()> {
    let root = repository_root();
    let documents = documents(&root)?;

    // A rule that scans nothing passes, and would keep passing after someone
    // moved the documentation elsewhere.
    assert!(
        documents.len() >= 20,
        "expected the top level and docs/ to hold the documentation, found {} files",
        documents.len()
    );

    let mut broken = Vec::new();
    for document in &documents {
        let markdown = fs::read_to_string(document)?;
        let directory = document.parent().unwrap_or(&root);
        for target in link_targets(&markdown) {
            let resolved = directory.join(&target);
            if !resolved.exists() {
                let name = document
                    .strip_prefix(&root)
                    .unwrap_or(document)
                    .display()
                    .to_string();
                broken.push(format!("{name}: {target}"));
            }
        }
    }

    assert!(
        broken.is_empty(),
        "{} link(s) point at files that are not in the tree:\n  {}",
        broken.len(),
        broken.join("\n  ")
    );
    Ok(())
}
