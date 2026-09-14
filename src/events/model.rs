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
    /// One kind of event. The journal holds 93 kinds and the page offered no
    /// way to pick one, so "show me every watchdog decision" meant scrolling
    /// or guessing a substring that happened to appear in the message.
    pub kind: Option<EventKind>,
    /// One node. Every other per-node surface accepts `?node=`; the journal did
    /// not, so the node page's "view the full journal" link dumped the whole
    /// workspace's history and left the operator to find their node in it.
    pub node_id: Option<String>,
}

impl RuntimeEventFilter {
    pub fn new(severity: Option<EventSeverity>, query: impl Into<String>, limit: usize) -> Self {
        Self {
            severity,
            query: query.into(),
            limit,
            kind: None,
            node_id: None,
        }
    }

    /// Every event of one kind, newest first.
    pub fn of_kind(kind: EventKind, limit: usize) -> Self {
        Self {
            kind: Some(kind),
            ..Self::new(None, "", limit)
        }
    }

    #[must_use]
    pub fn for_node(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }

    #[must_use]
    pub fn of(mut self, kind: Option<EventKind>) -> Self {
        self.kind = kind;
        self
    }
}

impl Default for RuntimeEventFilter {
    fn default() -> Self {
        Self {
            severity: None,
            query: String::new(),
            limit: 120,
            kind: None,
            node_id: None,
        }
    }
}
