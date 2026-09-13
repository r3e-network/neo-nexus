use super::*;

#[test]
fn api_tokens_cli_lifecycle_round_trip() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    drop(repository);

    let db_arg = db_path.display().to_string();

    // 1. Initially empty list
    let list_empty = action_from_args(["neo-nexus", "--list-api-tokens", &db_arg])?;
    match list_empty {
        CliAction::PrintWithExitCode { text, exit_code } => {
            assert_eq!(exit_code, 0);
            assert!(text.contains("No API tokens in workspace"));
        }
        other => anyhow::bail!("expected PrintWithExitCode, got {other:?}"),
    }

    // 2. Create token
    let create_action = action_from_args([
        "neo-nexus",
        "--create-api-token",
        &db_arg,
        "ci-builder",
        "read_fleet,read_readiness",
    ])?;
    match create_action {
        CliAction::PrintWithExitCode { text, exit_code } => {
            assert_eq!(exit_code, 0);
            assert!(text.contains("Created API token 'ci-builder'"));
            assert!(text.contains("read_fleet, read_readiness"));
            assert!(text.contains("Bearer Token"));
        }
        other => anyhow::bail!("expected PrintWithExitCode, got {other:?}"),
    }

    // 3. List table contains created token
    let list_action = action_from_args(["neo-nexus", "--list-api-tokens", &db_arg])?;
    match list_action {
        CliAction::PrintWithExitCode { text, exit_code } => {
            assert_eq!(exit_code, 0);
            assert!(text.contains("TOKEN ID"));
            assert!(text.contains("ci-builder"));
        }
        other => anyhow::bail!("expected PrintWithExitCode, got {other:?}"),
    }

    // 4. List json returns valid array
    let list_json = action_from_args(["neo-nexus", "--list-api-tokens-json", &db_arg])?;
    let token_id = match list_json {
        CliAction::PrintWithExitCode { text, exit_code } => {
            assert_eq!(exit_code, 0);
            let val: serde_json::Value = serde_json::from_str(&text)?;
            let arr = val.as_array().expect("array");
            assert_eq!(arr.len(), 1);
            assert_eq!(arr[0]["name"], "ci-builder");
            arr[0]["id"].as_str().unwrap().to_string()
        }
        other => anyhow::bail!("expected PrintWithExitCode, got {other:?}"),
    };

    // 5. Revoke token
    let revoke_action = action_from_args(["neo-nexus", "--revoke-api-token", &db_arg, &token_id])?;
    match revoke_action {
        CliAction::PrintWithExitCode { text, exit_code } => {
            assert_eq!(exit_code, 0);
            assert!(text.contains("Revoked API token"));
        }
        other => anyhow::bail!("expected PrintWithExitCode, got {other:?}"),
    }

    // 6. List is empty again
    let list_after = action_from_args(["neo-nexus", "--list-api-tokens", &db_arg])?;
    match list_after {
        CliAction::PrintWithExitCode { text, exit_code } => {
            assert_eq!(exit_code, 0);
            assert!(text.contains("No API tokens in workspace"));
        }
        other => anyhow::bail!("expected PrintWithExitCode, got {other:?}"),
    }

    Ok(())
}
