mod logging;
pub(super) mod model;
mod process;
mod termination;

use crate::logs::observations::LogObservations;
use std::sync::LazyLock;

pub use logging::log_path_for;
pub use model::{
    LaunchConfirmation, ManagedProcessKind, ManagedProcessSpec, ProcessExit, ProcessStart,
    ProcessStop,
};
pub use process::{ProcessSupervisor, LAUNCH_SETTLE_WINDOW};

pub fn log_observations() -> &'static LogObservations {
    static OBS: LazyLock<LogObservations> = LazyLock::new(LogObservations::default);
    &OBS
}

pub use termination::{live_pids, process_is_live, recorded_process, PidStop, RecordedProcess};
