use super::{EventKind, EventSeverity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRuntimeEvent {
    pub node_id: Option<String>,
    pub node_name: Option<String>,
    pub kind: EventKind,
    pub severity: EventSeverity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEvent {
    pub id: i64,
    pub occurred_at_unix: u64,
    pub node_id: Option<String>,
    pub node_name: Option<String>,
    pub kind: EventKind,
    pub severity: EventSeverity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEventFilter {
    pub severity: Option<EventSeverity>,
    pub query: String,
    pub limit: usize,
}

impl RuntimeEventFilter {
    pub fn new(severity: Option<EventSeverity>, query: impl Into<String>, limit: usize) -> Self {
        Self {
            severity,
            query: query.into(),
            limit,
        }
    }
}

impl Default for RuntimeEventFilter {
    fn default() -> Self {
        Self {
            severity: None,
            query: String::new(),
            limit: 120,
        }
    }
}
