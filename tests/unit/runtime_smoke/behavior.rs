use std::{path::Path, time::Duration};

use anyhow::Result;

use crate::types::NodeType;

use super::super::{smoke_runtime_command, RuntimeSmokeBinaryEvidenceStatus, RuntimeSmokeStatus};
use super::fake::{fake_binary_path, fake_runtime_command, write_fake_binary, FakeBehavior};

#[test]
fn runtime_smoke_passes_when_probe_identifies_neo_go() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let script = fake_binary_path(temp_dir.path(), "neo-go");
    write_fake_binary(
        &script,
        FakeBehavior::PrintAndExit("neo-go version 0.110.0", 0),
    )?;
    let (binary, args) = fake_runtime_command(&script);

    let report = smoke_runtime_command(NodeType::NeoGo, &binary, &args, Duration::from_secs(3));

    assert_eq!(report.status, RuntimeSmokeStatus::Passed);
    assert!(!report.attempts.is_empty());
    Ok(())
}

#[test]
fn runtime_smoke_passes_when_probe_identifies_neo_rs_node() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let script = fake_binary_path(temp_dir.path(), "neo-node");
    write_fake_binary(
        &script,
        FakeBehavior::PrintAndExit("neo-rs neo-node version 0.1.0", 0),
    )?;
    let (binary, args) = fake_runtime_command(&script);

    let report = smoke_runtime_command(NodeType::NeoRs, &binary, &args, Duration::from_secs(3));

    assert_eq!(report.status, RuntimeSmokeStatus::Passed);
    assert!(report.message.contains("neo-rs neo-node version"));
    assert_eq!(
        report.binary_evidence.status,
        RuntimeSmokeBinaryEvidenceStatus::Verified
    );
    assert!(report.binary_evidence.runtime_path.ends_with("neo-node"));
    assert!(report.binary_evidence.bytes.unwrap_or_default() > 0);
    assert_eq!(
        report.binary_evidence.sha256.as_deref().map(str::len),
        Some(64)
    );
    assert!(report.to_cli_text().contains("runtime-binary-sha256:"));
    Ok(())
}

#[test]
fn runtime_smoke_blocks_missing_binary_before_spawn() {
    let report = smoke_runtime_command(
        NodeType::NeoRs,
        Path::new("/definitely/missing/neo-node"),
        &[],
        Duration::from_secs(1),
    );

    assert_eq!(report.status, RuntimeSmokeStatus::Blocked);
    assert!(report.attempts.is_empty());
    assert_eq!(
        report.binary_evidence.status,
        RuntimeSmokeBinaryEvidenceStatus::Unavailable
    );
}

#[test]
fn runtime_smoke_times_out_hanging_probe() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let script = fake_binary_path(temp_dir.path(), "neo-node");
    write_fake_binary(&script, FakeBehavior::Sleep)?;
    let (binary, args) = fake_runtime_command(&script);

    let report = smoke_runtime_command(NodeType::NeoRs, &binary, &args, Duration::from_millis(80));

    assert_eq!(report.status, RuntimeSmokeStatus::TimedOut);
    assert!(report.attempts.iter().any(|attempt| attempt.timed_out));
    Ok(())
}

#[test]
fn runtime_smoke_reviews_successful_unidentified_output() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let script = fake_binary_path(temp_dir.path(), "neo-node");
    write_fake_binary(
        &script,
        FakeBehavior::PrintAndExit("custom runtime help", 0),
    )?;
    let (binary, args) = fake_runtime_command(&script);

    let report = smoke_runtime_command(NodeType::NeoRs, &binary, &args, Duration::from_secs(3));

    assert_eq!(report.status, RuntimeSmokeStatus::Review);
    assert!(report.status.is_success());
    Ok(())
}

#[test]
fn runtime_smoke_redacts_sensitive_attempt_evidence() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let script = fake_binary_path(temp_dir.path(), "neo-node");
    write_fake_binary(
        &script,
        FakeBehavior::PrintStdoutStderrAndExit(
            "neo-node Authorization: Bearer stdout-token api_key:stdout-key",
            "warning seed=stderr-seed webhook=https://hooks.example/raw-secret",
            0,
        ),
    )?;
    let (binary, mut args) = fake_runtime_command(&script);
    args.extend([
        "--api-key".to_string(),
        "argv-key".to_string(),
        "--wallet-password=argv-password".to_string(),
    ]);

    let report = smoke_runtime_command(NodeType::NeoRs, &binary, &args, Duration::from_secs(3));

    assert_eq!(report.status, RuntimeSmokeStatus::Passed);
    let text = report.to_cli_text();
    let json = serde_json::to_string(&report)?;
    for leaked in [
        "argv-key",
        "argv-password",
        "stdout-token",
        "stdout-key",
        "stderr-seed",
        "hooks.example",
        "raw-secret",
    ] {
        assert!(!text.contains(leaked), "{leaked} leaked in CLI text");
        assert!(!json.contains(leaked), "{leaked} leaked in JSON");
        assert!(
            !report.attempts[0].command_line.contains(leaked),
            "{leaked} leaked in command line"
        );
        assert!(
            !report.attempts[0].stdout.contains(leaked),
            "{leaked} leaked in stdout"
        );
        assert!(
            !report.attempts[0].stderr.contains(leaked),
            "{leaked} leaked in stderr"
        );
    }
    assert!(text.contains("<redacted>"));
    assert!(json.contains("<redacted>"));
    Ok(())
}
