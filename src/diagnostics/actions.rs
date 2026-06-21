use super::{
    resolution_counts::{empty_resolution_counts, increment_resolution_count},
    severity_counts::{empty_severity_counts, increment_severity_count},
    CheckSeverity, DiagnosticResolution, FleetDiagnostics,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReadinessActionFilter {
    pub severity: Option<CheckSeverity>,
    pub query: String,
    pub resolution: Option<DiagnosticResolution>,
}

impl ReadinessActionFilter {
    pub fn new(severity: Option<CheckSeverity>, query: impl Into<String>) -> Self {
        Self {
            severity,
            query: query.into(),
            resolution: None,
        }
    }

    pub fn with_resolution(mut self, resolution: Option<DiagnosticResolution>) -> Self {
        self.resolution = resolution;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessAction {
    pub node_id: String,
    pub node_name: String,
    pub node_score: usize,
    pub severity: CheckSeverity,
    pub title: String,
    pub detail: String,
    pub resolution: DiagnosticResolution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessActionKey {
    pub node_id: String,
    pub severity: CheckSeverity,
    pub title: String,
    pub detail: String,
    pub resolution: DiagnosticResolution,
}

impl ReadinessAction {
    pub fn key(&self) -> ReadinessActionKey {
        ReadinessActionKey {
            node_id: self.node_id.clone(),
            severity: self.severity,
            title: self.title.clone(),
            detail: self.detail.clone(),
            resolution: self.resolution,
        }
    }
}

impl ReadinessActionKey {
    pub fn matches(&self, action: &ReadinessAction) -> bool {
        self.node_id == action.node_id
            && self.severity == action.severity
            && self.title == action.title
            && self.detail == action.detail
            && self.resolution == action.resolution
    }
}

pub fn filter_readiness_actions(
    diagnostics: &FleetDiagnostics,
    filter: &ReadinessActionFilter,
) -> Vec<ReadinessAction> {
    let query = filter.query.trim().to_lowercase();
    let mut actions = diagnostics
        .nodes
        .iter()
        .flat_map(|node| {
            node.checks.iter().filter_map(|check| {
                if !is_actionable(check.severity) {
                    return None;
                }
                Some(ReadinessAction {
                    node_id: node.node_id.clone(),
                    node_name: node.node_name.clone(),
                    node_score: node.score,
                    severity: check.severity,
                    title: check.title.to_string(),
                    detail: check.detail.clone(),
                    resolution: check.resolution,
                })
            })
        })
        .filter(|action| {
            filter
                .severity
                .is_none_or(|severity| action.severity == severity)
        })
        .filter(|action| {
            filter
                .resolution
                .is_none_or(|resolution| action.resolution == resolution)
        })
        .filter(|action| query.is_empty() || action_matches(action, &query))
        .collect::<Vec<_>>();
    actions.sort_by(action_order);
    actions
}

pub fn readiness_action_resolution_counts(
    diagnostics: &FleetDiagnostics,
    filter: &ReadinessActionFilter,
) -> Vec<(DiagnosticResolution, usize)> {
    let query = filter.query.trim().to_lowercase();
    let mut counts = empty_resolution_counts();
    for node in &diagnostics.nodes {
        for check in &node.checks {
            if !is_actionable(check.severity)
                || filter
                    .severity
                    .is_some_and(|severity| check.severity != severity)
            {
                continue;
            }
            let action = ReadinessAction {
                node_id: node.node_id.clone(),
                node_name: node.node_name.clone(),
                node_score: node.score,
                severity: check.severity,
                title: check.title.to_string(),
                detail: check.detail.clone(),
                resolution: check.resolution,
            };
            if query.is_empty() || action_matches(&action, &query) {
                increment_resolution_count(&mut counts, check.resolution);
            }
        }
    }
    counts
}

pub fn readiness_action_severity_counts(
    diagnostics: &FleetDiagnostics,
    filter: &ReadinessActionFilter,
) -> Vec<(CheckSeverity, usize)> {
    let count_filter =
        ReadinessActionFilter::new(None, filter.query.as_str()).with_resolution(filter.resolution);
    let mut counts = empty_severity_counts();
    for action in filter_readiness_actions(diagnostics, &count_filter) {
        increment_severity_count(&mut counts, action.severity);
    }
    counts
}

fn is_actionable(severity: CheckSeverity) -> bool {
    matches!(severity, CheckSeverity::Critical | CheckSeverity::Warning)
}

fn action_order(left: &ReadinessAction, right: &ReadinessAction) -> std::cmp::Ordering {
    right
        .severity
        .cmp(&left.severity)
        .then_with(|| left.node_score.cmp(&right.node_score))
        .then_with(|| left.node_name.cmp(&right.node_name))
        .then_with(|| left.title.cmp(&right.title))
}

fn action_matches(action: &ReadinessAction, query: &str) -> bool {
    text_matches(&action.node_id, query)
        || text_matches(&action.node_name, query)
        || text_matches(action.severity.label(), query)
        || text_matches(&action.title, query)
        || text_matches(&action.detail, query)
        || action.resolution.matches_query(query)
        || text_matches(&action.node_score.to_string(), query)
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
#[path = "../../tests/unit/diagnostics/actions/tests.rs"]
mod tests;
