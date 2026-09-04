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
            "usage: neo-nexus {option} <neo-cli|neo-go|neo-rs|neox-geth|neox-rs> <binary> [runtime-args...]"
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
    if args.len() < 3 {
        anyhow::bail!("{option} is missing required arguments; run neo-nexus --help for usage");
    }
    if args.len() > 4 {
        anyhow::bail!("{option} does not accept extra arguments");
    }
    // A bare endpoint cannot be told apart by probing, so the family is an
    // argument: the N3 default keeps every existing invocation correct, and a
    // Neo X endpoint probed as N3 would misreport an outage.
    let family = match args.get(3).map(String::as_str) {
        Some(slug) => ChainFamily::from_slug(slug).with_context(|| {
            format!("{option}: unknown chain family '{slug}'; use neo-n3 or neo-x")
        })?,
        None => ChainFamily::NeoN3,
    };
    Ok(probe_rpc_endpoint_for(
        family,
        &args[2],
        Duration::from_secs(3),
    ))
}
