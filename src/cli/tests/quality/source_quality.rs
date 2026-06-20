use super::super::*;

#[test]
fn source_quality_cli_reports_clean_rust_sources() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    std::fs::write(
        temp_dir.path().join("src").join("lib.rs"),
        "pub fn checked(value: Option<u8>) -> bool { value.is_some() }\n",
    )?;

    let root_arg = temp_dir.path().display().to_string();
    let action = action_from_args(["neo-nexus", "--source-quality", &root_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected source quality action");
    };

    assert_eq!(exit_code, 0);
    assert!(text.contains("source-quality: ok"));
    assert!(text.contains("findings: 0"));
    assert!(text.contains("finding: none"));
    Ok(())
}

#[test]
fn source_quality_json_cli_rejects_panic_oriented_markers() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    let name = ["un", "wrap"].join("");
    let token = format!("{name}{}", '(');
    std::fs::write(
        temp_dir.path().join("src").join("lib.rs"),
        format!("pub fn unchecked(value: Option<u8>) -> u8 {{ value.{token}) }}\n"),
    )?;

    let root_arg = temp_dir.path().display().to_string();
    let action = action_from_args(["neo-nexus", "--source-quality-json", &root_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected source quality JSON action");
    };

    assert_eq!(exit_code, 1);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "failed");
    assert_eq!(value["success"], false);
    assert_eq!(value["report"]["status"], "failed");
    assert_eq!(value["report"]["finding_count"], 1);
    assert_eq!(value["report"]["findings"][0]["path"], "src/lib.rs");
    assert_eq!(value["report"]["findings"][0]["marker"], token);
    Ok(())
}
