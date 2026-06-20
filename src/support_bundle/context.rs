use std::{
    collections::BTreeMap,
    path::Path,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};

use crate::{
    diagnostics::{evaluate_fleet, FleetDiagnostics},
    event_journal_report::{event_export_filter, EventJournalReport, DEFAULT_EVENT_EXPORT_LIMIT},
    metrics::{MetricsCollector, MetricsSnapshot},
    readiness_report::WorkspaceReadinessReport,
    repository::Repository,
    types::{NodeConfig, NodeStatus},
    workspace_integrity::{WorkspaceIntegrityChecker, WorkspaceIntegrityReport},
};

use super::{
    logs::{default_log_dir, log_diagnosis_report},
    SupportBundleLogDiagnosisReport,
};

pub(super) struct SupportBundleContext {
    pub(super) application_version: String,
    pub(super) generated_at_unix: u64,
    pub(super) database: String,
    pub(super) nodes: Vec<NodeConfig>,
    pub(super) diagnostics: FleetDiagnostics,
    pub(super) readiness_report: WorkspaceReadinessReport,
    pub(super) integrity_report: WorkspaceIntegrityReport,
    pub(super) event_report: EventJournalReport,
    pub(super) log_diagnosis_report: SupportBundleLogDiagnosisReport,
    pub(super) metrics_snapshot: MetricsSnapshot,
    pub(super) running_nodes: usize,
    pub(super) matched_event_count: usize,
}

impl SupportBundleContext {
    pub(super) fn collect(
        repository: &Repository,
        database: &Path,
        application_version: String,
        generated_at_unix: u64,
    ) -> Result<Self> {
        let nodes = repository
            .list_nodes()
            .with_context(|| format!("failed to load nodes from {}", database.display()))?;
        let plugin_states = plugin_states_by_node(repository, &nodes)?;
        let diagnostics = evaluate_fleet(&nodes, &plugin_states);
        let readiness_report = WorkspaceReadinessReport::from_diagnostics(
            database,
            &diagnostics,
            application_version.clone(),
            generated_at_unix,
        );
        let integrity_report = WorkspaceIntegrityChecker::check_at(
            database,
            application_version.clone(),
            generated_at_unix,
        )?;
        let event_filter = event_export_filter(DEFAULT_EVENT_EXPORT_LIMIT, None, "");
        let matched_event_count = repository
            .count_events(&event_filter)
            .context("failed to count runtime events for support bundle")?;
        let events = repository
            .list_events(event_filter.clone())
            .context("failed to load runtime events for support bundle")?;
        let event_report = EventJournalReport::from_events(
            database,
            events,
            matched_event_count,
            &event_filter,
            application_version.clone(),
            generated_at_unix,
        );
        let log_dir = default_log_dir(database);
        let log_diagnosis_report = log_diagnosis_report(&nodes, &log_dir)?;
        let mut metrics_collector = MetricsCollector::new(Duration::ZERO);
        let metrics_snapshot = metrics_collector.refresh(&nodes, Instant::now());
        let running_nodes = nodes
            .iter()
            .filter(|node| node.status == NodeStatus::Running)
            .count();

        Ok(Self {
            application_version,
            generated_at_unix,
            database: database.display().to_string(),
            nodes,
            diagnostics,
            readiness_report,
            integrity_report,
            event_report,
            log_diagnosis_report,
            metrics_snapshot,
            running_nodes,
            matched_event_count,
        })
    }
}

fn plugin_states_by_node(
    repository: &Repository,
    nodes: &[NodeConfig],
) -> Result<BTreeMap<String, Vec<crate::catalog::PluginState>>> {
    nodes
        .iter()
        .map(|node| {
            repository
                .list_plugin_states(&node.id)
                .map(|states| (node.id.clone(), states))
                .with_context(|| format!("failed to load plugin state for {}", node.name))
        })
        .collect()
}
