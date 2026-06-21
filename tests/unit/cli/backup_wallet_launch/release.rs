use super::super::*;

#[test]
fn release_package_verification_cli_reports_valid_dist() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let binary_path = temp_dir.path().join("neo-nexus-test");
    std::fs::write(&binary_path, b"native binary bytes")?;
    let output_dir = temp_dir.path().join("dist");
    let package = ReleasePackager::package_binary(
        &binary_path,
        &output_dir,
        "9.8.7",
        ReleasePackagePlatform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        },
    )?;

    let output_arg = output_dir.display().to_string();
    let action = action_from_args(["neo-nexus", "--verify-release-package", &output_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected release package verification action");
    };
    assert_eq!(exit_code, 0);
    assert!(text.contains("release-package-verification: ok"));
    assert!(text.contains(&package.package_id));
    assert!(text.contains(&package.archive_sha256));
    Ok(())
}

#[test]
fn release_package_verification_json_cli_reports_success_and_failure() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let binary_path = temp_dir.path().join("neo-nexus-test");
    std::fs::write(&binary_path, b"native binary bytes")?;
    let output_dir = temp_dir.path().join("dist");
    let package = ReleasePackager::package_binary(
        &binary_path,
        &output_dir,
        "9.8.7",
        ReleasePackagePlatform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        },
    )?;

    let output_arg = output_dir.display().to_string();
    let action = action_from_args(["neo-nexus", "--verify-release-package-json", &output_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected JSON release package verification action");
    };
    assert_eq!(exit_code, 0);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "ok");
    assert_eq!(value["verification"]["package_id"], package.package_id);
    assert_eq!(
        value["verification"]["archive_sha256"],
        package.archive_sha256
    );

    std::fs::write(
        &package.checksum_path,
        format!(
            "0000000000000000000000000000000000000000000000000000000000000000  {}\n",
            package
                .archive_path
                .file_name()
                .context("missing archive name")?
                .to_string_lossy()
        ),
    )?;
    let failed = action_from_args(["neo-nexus", "--verify-release-package-json", &output_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = failed else {
        anyhow::bail!("expected failed JSON release package verification action");
    };
    assert_eq!(exit_code, 1);
    let failed_value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(failed_value["schema_version"], 1);
    assert_eq!(failed_value["status"], "failed");
    assert!(failed_value["message"]
        .as_str()
        .is_some_and(|message| message.contains("checksum SHA-256 mismatch")));
    Ok(())
}
