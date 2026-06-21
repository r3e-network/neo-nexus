use anyhow::Result;

use crate::cli::{self, CliAction};

use super::ManagerAction;

pub fn action_from_args<I, S>(args: I) -> Result<ManagerAction>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    Ok(match cli::action_from_args(args)? {
        CliAction::RunGui => ManagerAction::LaunchGui,
        CliAction::Print(text) => ManagerAction::WriteCli { text, exit_code: 0 },
        CliAction::PrintWithExitCode { text, exit_code } => {
            ManagerAction::WriteCli { text, exit_code }
        }
    })
}
