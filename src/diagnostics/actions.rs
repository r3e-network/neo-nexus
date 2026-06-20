use super::{CheckSeverity, FleetDiagnostics};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReadinessActionFilter {
    pub severity: Option<CheckSeverity>,
    pub query: String,
}

impl ReadinessActionFilter {
    pub fn new(severity: Option<CheckSeverity>, query: impl Into<String>) -> Self {
        Self {
            severity,
            query: query.into(),
        }
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
                })
            })
        })
        .filter(|action| {
            filter
                .severity
                .is_none_or(|severity| action.severity == severity)
        })
        .filter(|action| query.is_empty() || action_matches(action, &query))
        .collect::<Vec<_>>();
    actions.sort_by(action_order);
    actions
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
        || text_matches(&action.node_score.to_string(), query)
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
mod tests;
