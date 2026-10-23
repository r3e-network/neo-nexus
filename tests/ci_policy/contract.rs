//! Rules that keep the promises the product makes in two places it cannot
//! enforce by itself: the platform a test runs on, and the endpoints it
//! documents.
//!
//! Both classes of drift are invisible on the machine that made them. A
//! fixture that spawns `/bin/sleep` passes on Linux and fails on Windows; an
//! endpoint that is documented but not routed passes every review and fails
//! the first time a script calls it.
//!
//! Both rules read the syntax tree rather than the text. The text of a
//! program is ambiguous in exactly the places where these promises break: a
//! comment between the path and the handler, a quoted string that is not a
//! path, an ordinary string where a raw one was expected.

mod inline_handlers;
mod platform;
mod routes;

use std::{
    fs,
    path::{Path, PathBuf},
};

use syn::visit::Visit;

fn manifest_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::canonicalize(manifest_dir).unwrap_or_else(|_| manifest_dir.to_path_buf())
}

/// A test that spawns a platform-specific executable by literal path passes on
/// one platform and fails on another, and the failure looks like a product
/// bug.
///
/// The suite already has the right answer: `tests/support/stub_runtime.rs`
/// compiles small stand-in binaries that behave the same everywhere. A
/// platform binary may still be selected at runtime — that is a legitimate
/// `cfg` branch — but it must never be written as a literal inside
/// `Command::new`, because that is the shape that hides the platform
/// dependency in plain sight.
///
/// The literal is allowed when a compile-time gate keeps it away from every
/// platform where the executable does not exist. `#[cfg(unix)]` around
/// `Command::new("sh")` is honest; the same call without a gate is a promise
/// the release matrix cannot keep.
#[test]
fn ci_policy_rejects_platform_specific_executables_spawned_by_literal() -> anyhow::Result<()> {
    let root = manifest_root();
    let mut files = Vec::new();
    for directory in ["src", "tests"] {
        collect_rust_files(&root.join(directory), &mut files)?;
    }
    files.sort();

    let mut offences = Vec::new();
    for file in &files {
        let source = fs::read_to_string(file)?;
        // A call that names `Command` always spells the name out, so a file
        // without it cannot hold one. Skipping the rest keeps the rule to
        // the files that can actually break a job.
        if !source.contains("Command") {
            continue;
        }
        let parsed = syn::parse_file(&source)
            .map_err(|error| anyhow::anyhow!("{}: {error}", file.display()))?;
        let mut visitor = platform::PlatformGateVisitor::new();
        visitor.visit_file(&parsed);
        for spawn in visitor.spawns() {
            let missing = platform::missing_platforms(&spawn.literal, spawn.scope);
            if missing == 0 {
                continue;
            }
            offences.push(format!(
                "{}:{}:{}: {:?} may be spawned on {}, but it does not exist on {}",
                relative(&root, file),
                spawn.line,
                spawn.column + 1,
                spawn.literal,
                platform::describe(spawn.scope),
                platform::describe(missing),
            ));
        }
    }

    assert!(
        offences.is_empty(),
        "a platform-specific executable is spawned by literal path:\n{}\n\
         Spawn a compiled stand-in from tests/support/stub_runtime.rs, or wrap the \
         call in a compile-time #[cfg] gate that excludes the platforms where the \
         executable does not exist.",
        offences.join("\n")
    );
    Ok(())
}

/// An endpoint that the documentation promises and the router does not serve
/// is a promise the product breaks at the first call.
///
/// The OpenAPI document and the agent handbook are the contracts a script is
/// written against, so every endpoint either of them lists has to exist, with
/// the method it promises. The comparison ignores the parameter names:
/// `/api/nodes/{node_id}/metrics` and `/api/nodes/{id}/metrics` are the same
/// route, and only one spelling can be right in both files.
#[test]
fn ci_policy_rejects_documented_endpoints_that_do_not_exist() -> anyhow::Result<()> {
    let root = manifest_root();

    let mut documented =
        routes::openapi_endpoints(&fs::read_to_string(root.join("docs/openapi.yaml"))?)?;
    documented.extend(routes::handbook_endpoints(
        &fs::read_to_string(root.join("docs/AGENT_API.md"))?,
        "docs/AGENT_API.md",
    ));
    assert!(
        !documented.is_empty(),
        "no endpoints were read from the documentation"
    );

    let mut files = Vec::new();
    collect_rust_files(&root.join("src"), &mut files)?;
    files.sort();
    let mut served = Vec::new();
    for file in &files {
        let source = fs::read_to_string(file)?;
        // A route registration always spells the method name out, so a file
        // without it cannot hold one.
        if !source.contains("route") {
            continue;
        }
        served.extend(routes::collect_routes(&source));
    }

    let missing = routes::undocumented_endpoints(&documented, &served);
    assert!(
        missing.is_empty(),
        "the documentation promises endpoints the router does not serve:\n{}\n\
         Route the path, or remove the promise from the document.",
        missing
            .iter()
            .map(|endpoint| format!(
                "    {}: {} {}",
                endpoint.origin, endpoint.method, endpoint.path
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(())
}

/// An inline event handler never runs under the content security policy this
/// product already serves, so a control that depends on one is a control that
/// does nothing.
///
/// The console sets `script-src 'sha256-<hash>'` with no `'unsafe-inline'`. A
/// browser refuses to compile `onclick="..."` against that policy: the handler
/// is skipped and the page still behaves as if the attribute were not there.
/// The page renders correctly, so no test that looks at the markup can catch
/// it — the difference is only visible in a browser.
///
/// Two shipped. One guarded the deletion of every log file in the workspace;
/// the other two left a checkbox and a set of preset cards inert. Each looked
/// fine and each was broken.
///
/// The rule reads the syntax tree, so a comment that names an attribute and a
/// sentence of prose about one are not offences. Only markup carrying an
/// actual event attribute is.
#[test]
fn ci_policy_rejects_inline_event_handlers_the_content_policy_blocks() -> anyhow::Result<()> {
    let root = manifest_root();
    let mut files = Vec::new();
    collect_rust_files(&root.join("src"), &mut files)?;
    files.sort();

    let mut offences = Vec::new();
    for file in &files {
        // The bundled script is a JavaScript string literal, and a delegated
        // listener inside it is the supported way to do this.
        if file.ends_with("assets/script.rs") {
            continue;
        }
        let source = fs::read_to_string(file)?;
        // Markup with an event attribute always spells the attribute out, so a
        // file without it cannot hold one.
        if !source.contains("onclick")
            && !source.contains("onchange")
            && !source.contains("onsubmit")
            && !source.contains("oninput")
        {
            continue;
        }
        let parsed = syn::parse_file(&source)
            .map_err(|error| anyhow::anyhow!("{}: {error}", file.display()))?;
        for handler in inline_handlers::find_inline_handlers(&parsed) {
            offences.push(format!(
                "{}:{}:{}: inline {} handler",
                relative(&root, file),
                handler.line,
                handler.column,
                handler.attribute,
            ));
        }
    }

    assert!(
        offences.is_empty(),
        "a control depends on an inline event handler, which the content security \
         policy blocks:\n{}\n\
         The browser skips the handler and the control does nothing. Listen for the \
         event from the bundled script and mark the element with a data-* attribute \
         instead.",
        offences.join("\n")
    );
    Ok(())
}

/// The path of a file relative to the workspace root, with `/` separators.
fn relative(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

fn collect_rust_files(root: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(())
}
