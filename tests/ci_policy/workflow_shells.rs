use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn workflow_root() -> PathBuf {
    env::var_os("NEO_NEXUS_WORKFLOW_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf())
}

fn step_block<'a>(workflow: &'a str, name: &str) -> &'a str {
    let marker = format!("      - name: {name}\n");
    let start = workflow
        .find(&marker)
        .unwrap_or_else(|| panic!("workflow step is missing: {name}"));
    let remainder = &workflow[start + marker.len()..];
    let end = remainder
        .find("\n      - name: ")
        .unwrap_or(remainder.len());
    &remainder[..end]
}

#[test]
fn platform_specific_workflow_steps_pin_their_shell() -> anyhow::Result<()> {
    let root = workflow_root();
    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))?;
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
        let block = step_block(&ci, name);
        assert!(
            block.contains("        if: runner.os == 'Windows'\n        shell: pwsh\n"),
            "Windows step {name:?} must explicitly use PowerShell"
        );
    }

    let benchmarks = fs::read_to_string(root.join(".github/workflows/benchmarks.yml"))?;
    let summary = step_block(&benchmarks, "Generate performance summary");
    assert!(
        summary.contains("        if: success() || failure()\n        shell: bash\n"),
        "the benchmark summary uses Bash syntax and must pin Bash on the Windows matrix"
    );
    Ok(())
}
