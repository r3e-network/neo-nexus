use super::*;

pub(in crate::cli::actions) fn runtime_smoke_text(args: &[String]) -> Result<String> {
    Ok(runtime_smoke_report(args)?.to_cli_text())
}

pub(in crate::cli::actions) fn runtime_smoke_json_action(args: &[String]) -> Result<CliAction> {
    let report = runtime_smoke_report(args)?;
    Ok(CliAction::PrintWithExitCode {
        text: runtime_smoke_json_text(&report)?,
        exit_code: if report.status.is_success() { 0 } else { 1 },
    })
}

fn runtime_smoke_report(args: &[String]) -> Result<RuntimeSmokeReport> {
    if args.len() < 4 {
        let option = args.get(1).map_or("--runtime-smoke", String::as_str);
        anyhow::bail!(
            "usage: neo-nexus {option} <neo-cli|neo-go|neo-rs> <binary> [runtime-args...]"
        );
    }
    let node_type = NodeType::from_str(&args[2])?;
    let binary_path = PathBuf::from(&args[3]);
    let runtime_args = args[4..].to_vec();
    Ok(smoke_runtime_command(
        node_type,
        &binary_path,
        &runtime_args,
        Duration::from_secs(3),
    ))
}

pub(in crate::cli::actions) fn rpc_health_text(args: &[String]) -> Result<String> {
    Ok(rpc_health_report(args)?.to_cli_text())
}

pub(in crate::cli::actions) fn rpc_health_json_action(args: &[String]) -> Result<CliAction> {
    let report = rpc_health_report(args)?;
    let success = report.status == RpcHealthStatus::Healthy;
    Ok(CliAction::PrintWithExitCode {
        text: rpc_health_json_text(&report)?,
        exit_code: if success { 0 } else { 1 },
    })
}

fn rpc_health_report(args: &[String]) -> Result<RpcHealthReport> {
    let option = args.get(1).map_or("--rpc-health", String::as_str);
    require_arg_count(args, 3, option)?;
    Ok(probe_rpc_endpoint(&args[2], Duration::from_secs(3)))
}
