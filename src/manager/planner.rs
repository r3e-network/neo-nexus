use anyhow::Result;

use crate::cli::{self, CliAction};

use super::ManagerAction;

enum ManagerDispatchMode {
    Gui,
    Cli,
}

pub fn action_from_args<I, S>(args: I) -> Result<ManagerAction>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_string())
        .collect::<Vec<_>>();
    match classify_manager_mode(&args)? {
        ManagerDispatchMode::Gui => Ok(ManagerAction::LaunchGui),
        ManagerDispatchMode::Cli => cli_action_to_manager_action(cli::action_from_args(&args)?),
    }
}

fn classify_manager_mode(args: &[String]) -> Result<ManagerDispatchMode> {
    if args.get(1).is_none() {
        return Ok(ManagerDispatchMode::Gui);
    }
    if args.get(1).is_some_and(|arg| arg == "--gui") {
        if args.len() == 2 {
            return Ok(ManagerDispatchMode::Gui);
        }
        anyhow::bail!("--gui does not accept extra arguments");
    }
    Ok(ManagerDispatchMode::Cli)
}

fn cli_action_to_manager_action(action: CliAction) -> Result<ManagerAction> {
    Ok(match action {
        CliAction::Print(text) => ManagerAction::WriteCli { text, exit_code: 0 },
        CliAction::PrintWithExitCode { text, exit_code } => {
            ManagerAction::WriteCli { text, exit_code }
        }
    })
}
