use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn workflow_root() -> PathBuf {
    env::var_os("NEO_NEXUS_WORKFLOW_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf())
}

/// Read a workflow with LF line endings.
///
/// `core.autocrlf=true` (the Windows default) checks these files out with CRLF,
/// while the step markers below are written against LF. Normalising here keeps
/// the gate identical on every host instead of failing only on Windows.
fn read_workflow(root: &Path, name: &str) -> anyhow::Result<String> {
    let raw = fs::read_to_string(root.join(".github/workflows").join(name))?;
    Ok(raw.replace("\r\n", "\n"))
}

/// The block of a workflow step, by the name the step carries.
///
/// A missing step is a defect in the workflow, not a reason to abort the
/// process: the caller turns it into a test failure with the step name in it.
fn step_block<'a>(workflow: &'a str, name: &str) -> anyhow::Result<&'a str> {
    let marker = format!("      - name: {name}\n");
    let start = workflow
        .find(&marker)
        .ok_or_else(|| anyhow::anyhow!("workflow step is missing: {name}"))?;
    let remainder = &workflow[start + marker.len()..];
    let end = remainder
        .find("\n      - name: ")
        .unwrap_or(remainder.len());
    Ok(&remainder[..end])
}

#[test]
fn platform_specific_workflow_steps_pin_their_shell() -> anyhow::Result<()> {
    let root = workflow_root();
    let ci = read_workflow(&root, "ci.yml")?;
    for name in [
        "Alert preview CLI (Windows)",
        "Runtime smoke JSON CLI (Windows)",
        "RPC health JSON CLI (Windows)",
        "Workspace readiness CLI (Windows)",
        "Neo-rs standalone config CLI (Windows)",
        "Backup validation CLI (Windows)",
        "Wallet validation CLI (Windows)",
        "Release binary smoke (Windows)",
        "Package release artifacts (Windows)",
        "Verify release package integrity only (Windows)",
    ] {
        let block = step_block(&ci, name)?;
        assert!(
            block.contains("        if: runner.os == 'Windows'\n        shell: pwsh\n"),
            "Windows step {name:?} must explicitly use PowerShell"
        );
    }

    let benchmarks = read_workflow(&root, "benchmarks.yml")?;
    let summary = step_block(&benchmarks, "Generate performance summary")?;
    assert!(
        summary.contains("        if: success() || failure()\n        shell: bash\n"),
        "the benchmark summary uses Bash syntax and must pin Bash on the Windows matrix"
    );
    Ok(())
}
