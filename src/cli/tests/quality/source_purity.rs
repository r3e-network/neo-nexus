use super::super::*;

#[test]
fn source_purity_cli_reports_pure_rust_tree() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n")?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    std::fs::write(
        temp_dir.path().join("src").join("lib.rs"),
        "pub fn ok() {}\n",
    )?;
    std::fs::create_dir_all(temp_dir.path().join("docs"))?;
    std::fs::write(
        temp_dir.path().join("docs").join("catalog.example.json"),
        "{}\n",
    )?;
    std::fs::create_dir_all(temp_dir.path().join("dist"))?;
    std::fs::write(temp_dir.path().join("dist").join("bundle.js"), "")?;

    let root_arg = temp_dir.path().display().to_string();
    let action = action_from_args(["neo-nexus", "--source-purity", &root_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected source purity action");
    };

    assert_eq!(exit_code, 0);
    assert!(text.contains("source-purity: pure-rust"));
    assert!(text.contains("disallowed: 0"));
    assert!(text.contains("finding: none"));
    Ok(())
}

#[test]
fn source_purity_json_cli_rejects_frontend_artifacts() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n")?;
    std::fs::write(temp_dir.path().join("package.json"), "{}\n")?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    std::fs::write(temp_dir.path().join("src").join("App.tsx"), "")?;

    let root_arg = temp_dir.path().display().to_string();
    let action = action_from_args(["neo-nexus", "--source-purity-json", &root_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected source purity JSON action");
    };

    assert_eq!(exit_code, 1);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "failed");
    assert_eq!(value["success"], false);
    assert_eq!(value["report"]["status"], "failed");
    assert_eq!(value["report"]["disallowed_count"], 2);
    let findings = value["report"]["disallowed_entries"]
        .as_array()
        .context("missing source purity findings")?;
    assert!(findings
        .iter()
        .any(|finding| finding["path"] == "package.json"
            && finding["category"] == "node-toolchain-manifest"));
    assert!(findings
        .iter()
        .any(|finding| finding["path"] == "src/App.tsx"
            && finding["category"] == "frontend-source-file"));
    Ok(())
}
