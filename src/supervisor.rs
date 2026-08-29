mod logging;
mod model;
mod process;
mod termination;

pub use logging::log_path_for;
pub use model::{ManagedProcessKind, ManagedProcessSpec, ProcessExit, ProcessStart, ProcessStop};
pub use process::ProcessSupervisor;
pub use termination::{live_pids, process_is_live, PidStop};
