use super::*;

pub(super) const EVENT_FILTER_WHERE_SQL: &str = "
    (?1 IS NULL OR severity = ?1)
    AND (
        ?2 = ''
        OR lower(coalesce(node_name, '') || ' ' || kind || ' ' || message) LIKE ?3
    )
    AND (?4 IS NULL OR kind = ?4)
    AND (?5 IS NULL OR node_id = ?5)
";

pub(super) struct EventFilterBinding {
    pub severity: Option<String>,
    pub query: String,
    pub pattern: String,
    pub kind: Option<String>,
    pub node_id: Option<String>,
    pub limit: i64,
}

impl EventFilterBinding {
    pub fn from_filter(filter: RuntimeEventFilter) -> Self {
        Self::from_filter_ref(&filter)
    }

    pub fn from_filter_ref(filter: &RuntimeEventFilter) -> Self {
        let query = filter.query.trim().to_ascii_lowercase();
        Self {
            severity: filter.severity.map(|severity| severity.to_string()),
            pattern: format!("%{query}%"),
            query,
            kind: filter.kind.map(|kind| kind.label().to_string()),
            // An empty string is not a node. Treating one as a filter would
            // return nothing and read as "this node has no history".
            node_id: filter
                .node_id
                .as_deref()
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(str::to_string),
            limit: filter.limit.clamp(1, 500) as i64,
        }
    }
}
