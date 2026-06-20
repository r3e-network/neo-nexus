use super::*;

pub(in crate::cli::actions) fn workspace_readiness_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--workspace-readiness")?;
    let db_path = PathBuf::from(&args[2]);
    let diagnostics = workspace_diagnostics(&db_path)?;
    let exit_code = workspace_readiness_exit_code(&diagnostics);
    Ok(CliAction::PrintWithExitCode {
        text: workspace_readiness_text(&db_path, &diagnostics),
        exit_code,
    })
}

pub(in crate::cli::actions) fn workspace_readiness_json_action(
    args: &[String],
) -> Result<CliAction> {
    require_arg_count(args, 3, "--workspace-readiness-json")?;
    let db_path = PathBuf::from(&args[2]);
    let diagnostics = workspace_diagnostics(&db_path)?;
    let exit_code = workspace_readiness_exit_code(&diagnostics);
    Ok(CliAction::PrintWithExitCode {
        text: workspace_readiness_json_text(&db_path, &diagnostics)?,
        exit_code,
    })
}

pub(in crate::cli::actions) fn workspace_metrics_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--workspace-metrics")?;
    let snapshot = workspace_metrics_snapshot(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: snapshot.exit_code(),
        text: snapshot.to_cli_text(),
    })
}

pub(in crate::cli::actions) fn workspace_metrics_json_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--workspace-metrics-json")?;
    let snapshot = workspace_metrics_snapshot(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: snapshot.exit_code(),
        text: workspace_metrics_json_text(&snapshot)?,
    })
}

pub(in crate::cli::actions) fn workspace_metrics_prometheus_action(
    args: &[String],
) -> Result<CliAction> {
    require_arg_count(args, 3, "--workspace-metrics-prometheus")?;
    let snapshot = workspace_metrics_snapshot(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: snapshot.exit_code(),
        text: snapshot.to_prometheus_text(),
    })
}

pub(in crate::cli::actions) fn workspace_integrity_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--workspace-integrity")?;
    let report = workspace_integrity_report(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: report.to_cli_text(),
    })
}

pub(in crate::cli::actions) fn workspace_integrity_json_action(
    args: &[String],
) -> Result<CliAction> {
    require_arg_count(args, 3, "--workspace-integrity-json")?;
    let report = workspace_integrity_report(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: report.to_json_text()?,
    })
}

pub(in crate::cli::actions) fn workspace_diagnostics(db_path: &Path) -> Result<FleetDiagnostics> {
    let repository = Repository::open(db_path)
        .with_context(|| format!("failed to open workspace database {}", db_path.display()))?;
    let nodes = repository
        .list_nodes()
        .with_context(|| format!("failed to load nodes from {}", db_path.display()))?;
    let plugin_states = nodes
        .iter()
        .map(|node| {
            repository
                .list_plugin_states(&node.id)
                .map(|states| (node.id.clone(), states))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    Ok(evaluate_fleet(&nodes, &plugin_states))
}

fn workspace_integrity_report(db_path: PathBuf) -> Result<WorkspaceIntegrityReport> {
    WorkspaceIntegrityChecker::check(db_path, env!("CARGO_PKG_VERSION"))
}

fn workspace_metrics_snapshot(db_path: PathBuf) -> Result<MetricsSnapshot> {
    let repository = Repository::open(&db_path)
        .with_context(|| format!("failed to open workspace database {}", db_path.display()))?;
    let nodes = repository
        .list_nodes()
        .with_context(|| format!("failed to load nodes from {}", db_path.display()))?;
    let mut collector = MetricsCollector::new(Duration::ZERO);
    Ok(collector.refresh(&nodes, Instant::now()))
}
