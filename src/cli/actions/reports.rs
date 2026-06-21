use super::*;

pub(in crate::cli::actions) fn export_event_journal_text(args: &[String]) -> Result<String> {
    if args.len() < 4 {
        anyhow::bail!("usage: neo-nexus --export-event-journal <neonexus.db> <output-dir> [limit] [info|warning|critical|all] [query...]");
    }
    let db_path = PathBuf::from(&args[2]);
    let output_dir = PathBuf::from(&args[3]);
    if !db_path.is_file() {
        anyhow::bail!(
            "workspace database {} does not exist; pass an existing neonexus.db",
            db_path.display()
        );
    }
    let filter = parse_event_export_filter(args)?;
    let repository = Repository::open(&db_path)
        .with_context(|| format!("failed to open workspace database {}", db_path.display()))?;
    let matched_event_count = repository
        .count_events(&filter)
        .with_context(|| format!("failed to count runtime events in {}", db_path.display()))?;
    let events = repository
        .list_events(filter.clone())
        .with_context(|| format!("failed to export runtime events from {}", db_path.display()))?;
    let export = EventJournalReporter::write(
        output_dir,
        &db_path,
        events,
        matched_event_count,
        &filter,
        env!("CARGO_PKG_VERSION"),
    )?;
    Ok(export.to_cli_text())
}

pub(in crate::cli::actions) fn export_support_bundle_text(args: &[String]) -> Result<String> {
    let export = export_support_bundle(args, "--export-support-bundle")?;
    Ok(export.to_cli_text())
}

pub(in crate::cli::actions) fn export_support_bundle_json_text(args: &[String]) -> Result<String> {
    let export = export_support_bundle(args, "--export-support-bundle-json")?;
    export.to_json_text()
}

pub(in crate::cli::actions) fn workspace_readiness_report_action(
    args: &[String],
) -> Result<CliAction> {
    require_arg_count(args, 4, "--export-readiness-report")?;
    let db_path = PathBuf::from(&args[2]);
    let diagnostics = workspace_diagnostics(&db_path)?;
    let export = WorkspaceReadinessReporter::write(
        PathBuf::from(&args[3]),
        &db_path,
        &diagnostics,
        env!("CARGO_PKG_VERSION"),
    )?;
    Ok(CliAction::PrintWithExitCode {
        text: export.to_cli_text(),
        exit_code: export.report.exit_code(),
    })
}

fn parse_event_export_filter(args: &[String]) -> Result<RuntimeEventFilter> {
    let requested_limit = match args.get(4) {
        Some(value) => parse_event_export_limit(value)?,
        None => DEFAULT_EVENT_EXPORT_LIMIT,
    };
    let severity = match args.get(5).map(|value| value.as_str()) {
        None | Some("all") => None,
        Some(value) => Some(
            EventSeverity::from_str(value)
                .with_context(|| format!("invalid event journal severity filter: {value}"))?,
        ),
    };
    let query = if args.len() > 6 {
        args[6..].join(" ")
    } else {
        String::new()
    };
    Ok(event_export_filter(requested_limit, severity, query))
}

fn parse_event_export_limit(value: &str) -> Result<usize> {
    let limit = value
        .parse::<usize>()
        .with_context(|| format!("invalid event journal export limit: {value}"))?;
    if !(1..=MAX_EVENT_EXPORT_LIMIT).contains(&limit) {
        anyhow::bail!("event journal export limit must be between 1 and {MAX_EVENT_EXPORT_LIMIT}");
    }
    Ok(limit)
}

fn export_support_bundle(args: &[String], option: &str) -> Result<WorkspaceSupportBundleExport> {
    require_arg_count(args, 4, option)?;
    let db_path = PathBuf::from(&args[2]);
    if !db_path.is_file() {
        anyhow::bail!(
            "workspace database {} does not exist; pass an existing neonexus.db",
            db_path.display()
        );
    }
    let repository = Repository::open(&db_path)
        .with_context(|| format!("failed to open workspace database {}", db_path.display()))?;
    WorkspaceSupportBundleExporter::write(
        &repository,
        &db_path,
        PathBuf::from(&args[3]),
        env!("CARGO_PKG_VERSION"),
    )
}
