mod logging;
mod model;
mod process;
mod termination;
#[cfg(windows)]
mod windows_console;

#[cfg(windows)]
#[doc(hidden)]
pub use windows_console::console_break_helper_from_args;

pub use logging::log_path_for;
pub use model::{ManagedProcessKind, ManagedProcessSpec, ProcessExit, ProcessStart, ProcessStop};
pub use process::ProcessSupervisor;
pub use termination::{live_pids, process_is_live, recorded_process, PidStop, RecordedProcess};
