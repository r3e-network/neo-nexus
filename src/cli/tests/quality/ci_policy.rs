use super::super::*;

#[test]
fn ci_policy_json_cli_reports_workflow_gaps() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let workflow_path = temp_dir.path().join("ci.yml");
    std::fs::write(
        &workflow_path,
        r#"
name: Partial CI

jobs:
  verify:
strategy:
  matrix:
    os: [ubuntu-latest]
steps:
  - uses: actions/setup-node@v4
  - run: npm run build
  - run: cargo check
"#,
    )?;

    let workflow_arg = workflow_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--ci-policy-json", &workflow_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected CI policy JSON action");
    };

    assert_eq!(exit_code, 1);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "failed");
    assert_eq!(value["success"], false);
    assert_eq!(value["report"]["status"], "failed");
    assert!(value["report"]["missing_os"]
        .as_array()
        .context("missing CI policy OS list")?
        .iter()
        .any(|os| os == "windows-latest"));
    assert!(value["report"]["findings"]
        .as_array()
        .context("missing CI policy findings")?
        .iter()
        .any(|finding| finding["category"] == "forbidden-ci-tooling"));
    Ok(())
}
