use anyhow::Context;

use super::{action_from_args, ManagerAction, ManagerMode};
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn manager_classifies_default_web_and_explicit_cli_modes() -> anyhow::Result<()> {
    let default = action_from_args(["neo-nexus"])?;
    assert!(matches!(default, ManagerAction::ServeWeb(_)));
    assert_eq!(default.mode(), ManagerMode::Web);

    let explicit_web = action_from_args(["neo-nexus", "--web"])?;
    assert!(matches!(explicit_web, ManagerAction::ServeWeb(_)));

    let help = action_from_args(["neo-nexus", "--help"])?;
    assert_eq!(help.mode(), ManagerMode::Cli);
    assert!(matches!(
        help,
        ManagerAction::WriteCli {
            text,
            exit_code: 0,
        } if text.contains("NeoNexus")
    ));
    Ok(())
}

#[test]
fn manager_preserves_cli_exit_code_without_gui_dependencies() -> anyhow::Result<()> {
    let root = std::env::temp_dir().join(format!(
        "neo-nexus-manager-source-purity-{}",
        std::process::id()
    ));
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir_all(&root)?;
    std::fs::write(root.join("package.json"), "{}")?;

    let action = action_from_args(vec![
        "neo-nexus".to_string(),
        "--source-purity".to_string(),
        root.to_str()
            .context("temporary source purity path is not valid UTF-8")?
            .to_string(),
    ])?;
    std::fs::remove_dir_all(&root)?;

    assert_eq!(action.mode(), ManagerMode::Cli);
    assert!(matches!(
        action,
        ManagerAction::WriteCli {
            text,
            exit_code: 1,
        } if text.contains("source-purity: failed")
    ));
    Ok(())
}

#[test]
fn manager_parses_web_launch_options() {
    let action = action_from_args([
        "neo-nexus",
        "--web",
        "--bind",
        "0.0.0.0",
        "--port",
        "9090",
        "--web-token",
        "ops-secret",
    ])
    .expect("web flags should parse");
    let ManagerAction::ServeWeb(launch) = action else {
        unreachable!("--web with bind/port/token parses to ServeWeb");
    };
    assert_eq!(launch.bind, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    assert_eq!(launch.port, 9090);
    assert_eq!(launch.token.as_deref(), Some("ops-secret"));
}

#[test]
fn manager_rejects_removed_gui_flag_and_bad_web_options() {
    assert!(action_from_args(["neo-nexus", "--gui"]).is_err());
    assert!(action_from_args(["neo-nexus", "--web", "--port", "not-a-port"]).is_err());
    assert!(action_from_args(["neo-nexus", "--web", "--bind"]).is_err());
}

#[test]
fn manager_planner_keeps_mode_classification_separate() -> anyhow::Result<()> {
    let planner_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/manager/planner.rs");
    let planner_source = std::fs::read_to_string(planner_path)?;

    assert!(planner_source.contains("fn classify_manager_mode"));
    assert!(planner_source.contains("match classify_manager_mode(&args)?"));
    Ok(())
}

#[test]
fn manager_cli_output_normalizes_text_and_exit_status() -> anyhow::Result<()> {
    let Some(output) = (ManagerAction::WriteCli {
        text: "workspace-ready".to_string(),
        exit_code: 0,
    })
    .into_cli_output() else {
        anyhow::bail!("write action should expose CLI output");
    };
    assert_eq!(output.text_with_trailing_newline(), "workspace-ready\n");
    assert_eq!(output.exit_code(), 0);
    assert!(!output.should_exit_process());

    let Some(failed_output) = (ManagerAction::WriteCli {
        text: "source-purity: failed\n".to_string(),
        exit_code: 1,
    })
    .into_cli_output() else {
        anyhow::bail!("failed write action should expose CLI output");
    };
    assert_eq!(
        failed_output.text_with_trailing_newline(),
        "source-purity: failed\n"
    );
    assert_eq!(failed_output.exit_code(), 1);
    assert!(failed_output.should_exit_process());
    let default = action_from_args(["neo-nexus"])?;
    assert!(default.into_cli_output().is_none());
    Ok(())
}
