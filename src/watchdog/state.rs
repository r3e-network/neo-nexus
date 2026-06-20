use std::time::Instant;

#[derive(Debug, Clone)]
pub(super) struct RestartState {
    pub(super) attempts: u32,
    pub(super) next_restart_at: Option<Instant>,
    pub(super) exhausted: bool,
}

impl RestartState {
    pub(super) fn new() -> Self {
        Self {
            attempts: 0,
            next_restart_at: None,
            exhausted: false,
        }
    }
}
