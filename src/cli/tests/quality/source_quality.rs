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
    assert!(text.contains("rust-files: 1"));
    assert!(text.contains("maintenance-files: 0"));
    assert!(text.contains("findings: 0"));
    assert!(text.contains("finding: none"));
    Ok(())
}

#[test]
fn source_quality_cli_reports_rust_and_maintenance_file_coverage() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    std::fs::write(
        temp_dir.path().join("src").join("lib.rs"),
        "pub fn checked(value: Option<u8>) -> bool { value.is_some() }\n",
    )?;
    std::fs::write(temp_dir.path().join("README.md"), "# NeoNexus\n")?;

    let root_arg = temp_dir.path().display().to_string();
    let action = action_from_args(["neo-nexus", "--source-quality", &root_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected source quality action");
    };

    assert_eq!(exit_code, 0);
    assert!(text.contains("scanned-files: 2"));
    assert!(text.contains("rust-files: 1"));
    assert!(text.contains("maintenance-files: 1"));
    Ok(())
}

#[test]
fn source_quality_cli_finding_text_includes_source_snippet() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("src").join("app"))?;
    let token = format!("{}{}", "Option", '+');
    let source_line = format!("fn label() -> &'static str {{ \"Next {token}Down\" }}");
    std::fs::write(
        temp_dir.path().join("src").join("app").join("menu.rs"),
        format!("{source_line}\n"),
    )?;

    let root_arg = temp_dir.path().display().to_string();
    let action = action_from_args(["neo-nexus", "--source-quality", &root_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected source quality action");
    };

    assert_eq!(exit_code, 1);
    assert!(text.contains("hardcoded-platform-shortcut-label"));
    assert!(text.contains(&token));
    assert!(text.contains(&format!("snippet: {source_line}")));
    assert!(text.contains("hint: generate shortcut labels through the platform formatter"));
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
    assert_eq!(value["report"]["rust_files"], 1);
    assert_eq!(value["report"]["maintenance_files"], 0);
    assert_eq!(value["report"]["finding_count"], 1);
    assert_eq!(value["report"]["findings"][0]["path"], "src/lib.rs");
    assert_eq!(value["report"]["findings"][0]["marker"], token);
    Ok(())
}

#[test]
fn source_quality_json_cli_rejects_hardcoded_platform_shortcut_labels() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("src").join("app"))?;
    let token = format!("{}{}", "Ctrl", '+');
    std::fs::write(
        temp_dir.path().join("src").join("app").join("menu.rs"),
        format!("fn label() -> &'static str {{ \"Reload {token}R\" }}\n"),
    )?;

    let root_arg = temp_dir.path().display().to_string();
    let action = action_from_args(["neo-nexus", "--source-quality-json", &root_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected source quality JSON action");
    };

    assert_eq!(exit_code, 1);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["report"]["finding_count"], 1);
    assert_eq!(value["report"]["findings"][0]["path"], "src/app/menu.rs");
    assert_eq!(
        value["report"]["findings"][0]["category"],
        "hardcoded-platform-shortcut-label"
    );
    assert_eq!(value["report"]["findings"][0]["marker"], token);
    assert_eq!(
        value["report"]["findings"][0]["hint"],
        "generate shortcut labels through the platform formatter"
    );
    Ok(())
}
