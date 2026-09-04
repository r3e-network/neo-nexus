mod model;
mod policy;
mod recovery;
mod scheduler;
mod state;

pub use model::{RestartAttempt, RestartOutcome, WatchdogStatus};
pub use policy::{
    default_restart_policy, RestartPolicy, DEFAULT_BASE_DELAY, DEFAULT_MAX_DELAY,
    DEFAULT_MAX_RESTART_ATTEMPTS,
};
pub(crate) use recovery::{unix_millis, RecoveryClaim, RecoveryState};
pub use scheduler::Watchdog;
