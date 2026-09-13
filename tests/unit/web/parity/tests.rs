//! Every control on a page has to reach a route that exists.
//!
//! The workspace-backup page's only button posted to `/backup/export`, and the
//! only registered route was `POST /backup`. Nothing caught it: the form
//! action is a string, the route table is a builder chain, and no test ever
//! pressed the button. A workspace backup therefore could not be taken from
//! the console at all — while the page described the round trip in prose.
//!
//! This is the smallest useful version of a parity gate. It reads the router
//! source and the page sources and checks that every form action written as a
//! plain string literal names a registered path.
//!
//! It deliberately does not try to resolve actions built with `format!`, which
//! carry interpolated ids — those are reported as a count so that shrinking
//! coverage is visible rather than silent.

use std::{collections::BTreeSet, fs, path::Path};

const ROUTER: &str = "src/web/router.rs";

/// Paths registered on the router, as written — `{id}` placeholders included.
fn registered_routes() -> BTreeSet<String> {
    let source = fs::read_to_string(ROUTER).expect("the router source is readable");
    let mut routes = BTreeSet::new();
    let mut rest = source.as_str();
    while let Some(start) = rest.find(".route(") {
        rest = &rest[start + ".route(".len()..];
        // The path is the next string literal, which may sit on the following
        // line or behind a comment.
        let Some(open) = rest.find('"') else { break };
        let Some(len) = rest[open + 1..].find('"') else {
            break;
        };
        let path = &rest[open + 1..open + 1 + len];
        if path.starts_with('/') {
            routes.insert(path.to_string());
        }
        rest = &rest[open + 1 + len..];
    }
    assert!(
        routes.len() > 20,
        "only {} routes parsed out of {ROUTER}; the parser has drifted from the source",
        routes.len()
    );
    routes
}

fn page_sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                if let Ok(text) = fs::read_to_string(&path) {
                    out.push((path.display().to_string(), text));
                }
            }
        }
    }
    let mut sources = Vec::new();
    walk(Path::new("src/web/pages"), &mut sources);
    sources
}

/// Form targets written as a plain path, with the file and line they sit on.
///
/// Two spellings appear in the pages: `action="/path"` inside a raw HTML
/// literal, and `html::control_form("/path", …)`.
fn literal_form_targets() -> Vec<(String, usize, String)> {
    let mut targets = Vec::new();
    for (path, text) in page_sources() {
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for marker in [r#"action=""#, r#"control_form(""#] {
                let mut rest = line;
                while let Some(at) = rest.find(marker) {
                    let after = &rest[at + marker.len()..];
                    if let Some(end) = after.find('"') {
                        let target = &after[..end];
                        // Interpolated targets are resolved at render time.
                        if target.starts_with('/') && !target.contains('{') {
                            targets.push((path.clone(), index + 1, target.to_string()));
                        }
                        rest = &after[end..];
                    } else {
                        break;
                    }
                }
            }
        }
    }
    targets
}

#[test]
fn every_literal_form_action_reaches_a_registered_route() {
    let routes = registered_routes();
    let targets = literal_form_targets();
    assert!(
        targets.len() > 10,
        "only {} literal form targets found; the scan has drifted from the pages",
        targets.len()
    );

    let unreachable: Vec<String> = targets
        .iter()
        .filter(|(_, _, target)| !routes.contains(target.as_str()))
        .map(|(file, line, target)| {
            format!("{file}:{line} posts to {target}, which is not a route")
        })
        .collect();

    assert!(
        unreachable.is_empty(),
        "a page submits to a path the router does not serve, so the control silently 404s:\n{}",
        unreachable.join("\n")
    );
}
