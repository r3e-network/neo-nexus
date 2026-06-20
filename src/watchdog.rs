mod model;
mod policy;
mod scheduler;
mod state;

pub use model::{RestartAttempt, RestartOutcome, WatchdogStatus};
pub use policy::{
    default_restart_policy, RestartPolicy, DEFAULT_BASE_DELAY, DEFAULT_MAX_DELAY,
    DEFAULT_MAX_RESTART_ATTEMPTS,
};
pub use scheduler::Watchdog;
