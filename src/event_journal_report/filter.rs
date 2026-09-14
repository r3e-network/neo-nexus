use crate::events::{EventSeverity, RuntimeEventFilter};

pub const DEFAULT_EVENT_EXPORT_LIMIT: usize = 250;
pub const MAX_EVENT_EXPORT_LIMIT: usize = 500;

pub fn event_export_filter(
    limit: usize,
    severity: Option<EventSeverity>,
    query: impl Into<String>,
) -> RuntimeEventFilter {
    RuntimeEventFilter::new(severity, query, limit.clamp(1, MAX_EVENT_EXPORT_LIMIT))
}

/// Clamp a filter an operator built on the page to what an export may carry.
///
/// The scope they chose is preserved — kind, node, severity, search — and only
/// the row count is bounded, so the file they receive is the journal they were
/// looking at rather than a different query with the same name.
pub fn export_scope(filter: &RuntimeEventFilter) -> RuntimeEventFilter {
    RuntimeEventFilter {
        limit: filter.limit.clamp(1, MAX_EVENT_EXPORT_LIMIT),
        ..filter.clone()
    }
}
