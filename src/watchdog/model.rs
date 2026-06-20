use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartOutcome {
    Disabled,
    Scheduled { attempt: u32, delay: Duration },
    Exhausted { attempts: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestartAttempt {
    pub node_id: String,
    pub attempt: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchdogStatus {
    Idle,
    Pending { attempt: u32, remaining: Duration },
    Exhausted { attempts: u32 },
}
