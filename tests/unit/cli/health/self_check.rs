use super::super::*;

#[test]
fn self_check_exercises_workspace_database() -> Result<()> {
    let action = action_from_args(["neo-nexus", "--self-check"])?;
    assert!(matches!(action, CliAction::Print(text) if text.contains("workspace-db: ok")));
    Ok(())
}
