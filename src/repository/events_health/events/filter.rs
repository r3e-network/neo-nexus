use super::*;

pub(super) const EVENT_FILTER_WHERE_SQL: &str = "
    (?1 IS NULL OR severity = ?1)
    AND (
        ?2 = ''
        OR lower(coalesce(node_name, '') || ' ' || kind || ' ' || message) LIKE ?3
    )
";

pub(super) struct EventFilterBinding {
    pub severity: Option<String>,
    pub query: String,
    pub pattern: String,
    pub limit: i64,
}

impl EventFilterBinding {
    pub fn from_filter(filter: RuntimeEventFilter) -> Self {
        let query = filter.query.trim().to_ascii_lowercase();
        Self {
            severity: filter.severity.map(|severity| severity.to_string()),
            pattern: format!("%{query}%"),
            query,
            limit: filter.limit.clamp(1, 500) as i64,
        }
    }

    pub fn from_filter_ref(filter: &RuntimeEventFilter) -> Self {
        let query = filter.query.trim().to_ascii_lowercase();
        Self {
            severity: filter.severity.map(|severity| severity.to_string()),
            pattern: format!("%{query}%"),
            query,
            limit: filter.limit.clamp(1, 500) as i64,
        }
    }
}
