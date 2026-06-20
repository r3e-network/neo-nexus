use crate::events::NewRuntimeEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreNodeOutcome {
    Created,
    Updated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSetting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoredRuntimeEvent {
    pub occurred_at_unix: u64,
    pub event: NewRuntimeEvent,
}
