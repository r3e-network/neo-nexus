use super::*;

fn agent_action(args: &[String], option: &str, action: &str) -> Result<CliAction> {
    require_arg_count(args, 4, option)?;
    let repository = Repository::open(PathBuf::from(&args[2]))
        .with_context(|| format!("failed to open workspace database {}", args[2]))?;
    let state = crate::supervision::EngineState {
        repository: repository.clone(),
        data_dir: repository
            .db_path()
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf),
        supervisor: std::sync::Arc::new(std::sync::Mutex::new(
            crate::supervisor::ProcessSupervisor::default(),
        )),
        heartbeat: crate::supervision_heartbeat::SupervisionHeartbeat::new(),
        notifications: crate::supervision_heartbeat::SupervisionHeartbeat::new(),
    };
    let id = &args[3];
    match action {
        "start" => crate::core::agents::start(&state, id)?,
        "stop" => crate::core::agents::stop(&state, id)?,
        "restart" => {
            crate::core::agents::stop(&state, id)?;
            crate::core::agents::start(&state, id)?;
        }
        _ => anyhow::bail!("unknown agent action"),
    }
    Ok(CliAction::Print(format!("agent {id} {action} requested\n")))
}

pub(super) fn agent_start_action(args: &[String]) -> Result<CliAction> {
    agent_action(args, "--agent-start", "start")
}

pub(super) fn agent_stop_action(args: &[String]) -> Result<CliAction> {
    agent_action(args, "--agent-stop", "stop")
}

pub(super) fn agent_restart_action(args: &[String]) -> Result<CliAction> {
    agent_action(args, "--agent-restart", "restart")
}

pub(super) fn agent_list_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--agent-list")?;
    let repository = Repository::open(PathBuf::from(&args[2]))
        .with_context(|| format!("failed to open workspace database {}", args[2]))?;
    let records = repository.list_agents()?;
    let text = records
        .iter()
        .map(|record| {
            format!(
                "{}\t{:?}\t{:?}\tpid={}\tdesired={}\n",
                record.profile.id,
                record.profile.kind,
                record.status,
                record
                    .pid
                    .map_or_else(|| "-".to_string(), |pid| pid.to_string()),
                record.desired_running
            )
        })
        .collect::<String>();
    Ok(CliAction::Print(text))
}
