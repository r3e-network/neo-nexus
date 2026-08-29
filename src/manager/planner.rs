use std::net::IpAddr;

use anyhow::{bail, Result};

use crate::cli::{self, CliAction};

use super::{ManagerAction, WebLaunch};

enum ManagerDispatchMode {
    Web(WebLaunch),
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
        ManagerDispatchMode::Web(launch) => Ok(ManagerAction::ServeWeb(launch)),
        ManagerDispatchMode::Cli => cli_action_to_manager_action(cli::action_from_args(&args)?),
    }
}

/// No options starts the web workbench — the default experience, cloud or
/// desktop. `--web` is the explicit spelling and accepts `--bind`, `--port`,
/// and `--web-token`. `--gui` was removed in 4.0.0; the error tells operators
/// where the workbench went.
fn classify_manager_mode(args: &[String]) -> Result<ManagerDispatchMode> {
    if args.get(1).is_none() {
        return Ok(ManagerDispatchMode::Web(default_launch()?));
    }
    match args[1].as_str() {
        "--web" => Ok(ManagerDispatchMode::Web(parse_web_launch(args)?)),
        "--gui" => bail!(
            "--gui was removed in 4.0.0 — the workbench is now the web surface; \
             run with no options or --web and open the printed address in a browser"
        ),
        _ => Ok(ManagerDispatchMode::Cli),
    }
}

fn default_launch() -> Result<WebLaunch> {
    Ok(WebLaunch {
        bind: IpAddr::from([127, 0, 0, 1]),
        port: 8080,
        token: None,
        data_dir: crate::web::server::default_data_dir(),
    })
}

fn parse_web_launch(args: &[String]) -> Result<WebLaunch> {
    let mut launch = default_launch()?;
    let mut index = 2;
    while index < args.len() {
        let Some(value) = args.get(index + 1) else {
            bail!("{} requires a value", args[index]);
        };
        match args[index].as_str() {
            "--bind" => launch.bind = value.parse::<IpAddr>()?,
            "--port" => launch.port = value.parse::<u16>()?,
            "--web-token" => launch.token = Some(value.clone()),
            other => bail!("unknown option {other} after --web"),
        }
        index += 2;
    }
    Ok(launch)
}

fn cli_action_to_manager_action(action: CliAction) -> Result<ManagerAction> {
    Ok(match action {
        CliAction::Print(text) => ManagerAction::WriteCli { text, exit_code: 0 },
        CliAction::PrintWithExitCode { text, exit_code } => {
            ManagerAction::WriteCli { text, exit_code }
        }
    })
}
