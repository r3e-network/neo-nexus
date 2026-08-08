//! Supervisor process suites.
//!
//! Every test here spawns a real child and waits for it to exit, so the whole
//! group is unix-only. Gating the modules rather than each test inside them
//! keeps the reason in one place, and stops their `use super::*` from being an
//! unused import on the platforms that skip them.
#[cfg(unix)]
use crate::*;

#[cfg(unix)]
#[path = "processes/output.rs"]
mod output;
#[cfg(unix)]
#[path = "processes/reap.rs"]
mod reap;
#[cfg(unix)]
#[path = "processes/restart.rs"]
mod restart;
#[cfg(unix)]
#[path = "processes/sidecars.rs"]
mod sidecars;
#[cfg(unix)]
#[path = "processes/termination.rs"]
mod termination;
