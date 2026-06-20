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
