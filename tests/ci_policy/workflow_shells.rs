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

#[test]
fn advisory_audit_uses_cli_exit_status_without_check_run_permissions() -> anyhow::Result<()> {
    let ci = read_workflow(&workflow_root(), "ci.yml")?;
    let workflow: serde_yaml::Value = serde_yaml::from_str(&ci)?;
    let verify = &workflow["jobs"]["verify"];
    let steps = verify["steps"]
        .as_sequence()
        .expect("verify job must contain steps");
    let audit = steps
        .iter()
        .find(|step| step["name"].as_str() == Some("RustSec advisory audit"))
        .expect("the advisory audit must remain in the verify job");
    assert!(
        audit["uses"].is_null(),
        "audit must use the CLI exit status"
    );
    assert!(
        audit["with"].is_null(),
        "audit must not receive a GitHub token"
    );
    assert!(
        audit["continue-on-error"].is_null(),
        "audit failures must fail the job"
    );
    let commands = audit["run"].as_str().expect("audit commands are required");
    assert_eq!(
        commands.lines().collect::<Vec<_>>(),
        [
            "cargo install cargo-audit --version 0.22.2 --locked",
            "cargo audit --deny unsound",
        ],
        "audit must install the pinned tool and propagate vulnerability/unsound failures"
    );
    assert!(verify["permissions"]["checks"].is_null());
    assert!(verify["permissions"]["issues"].is_null());
    Ok(())
}

#[test]
fn yaml_parser_dependency_is_maintained_without_advisory_exemptions() -> anyhow::Result<()> {
    let root = workflow_root();
    let manifest: toml::Value = toml::from_str(&fs::read_to_string(root.join("Cargo.toml"))?)?;
    let parser = &manifest["dependencies"]["serde_yaml"];
    assert_eq!(
        parser.get("package").and_then(toml::Value::as_str),
        Some("yaml_serde")
    );
    assert_eq!(
        parser.get("version").and_then(toml::Value::as_str),
        Some("0.10.7")
    );
    let lock: toml::Value = toml::from_str(&fs::read_to_string(root.join("Cargo.lock"))?)?;
    for package in lock["package"].as_array().expect("lock packages") {
        assert!(
            !matches!(
                package["name"].as_str(),
                Some("unsafe-libyaml" | "serde_yaml")
            ),
            "retired YAML implementations must not remain in the resolved graph"
        );
    }
    let policy: toml::Value = toml::from_str(&fs::read_to_string(root.join("deny.toml"))?)?;
    assert!(policy["advisories"]["ignore"]
        .as_array()
        .expect("advisory policy")
        .is_empty());
    Ok(())
}
