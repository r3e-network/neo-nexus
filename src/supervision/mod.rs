//! The workbench's supervision engine: the loop that notices a node has died,
//! brings it back within policy, probes what the Settings page says to probe,
//! and routes the alerts the Alerts page says to route.
//!
//! Every one of those behaviours used to ride on the desktop shell's frame tick:
//! `src/app/frame.rs` drained probe results each frame, `rpc_health_flow` and
//! `remote_federation_flow` spawned probes on their policy intervals, and
//! `policy_alert_flow` delivered webhooks. Removing `src/app/` removed the
//! heartbeat but not the settings that describe it, so the workbench went on
//! offering policies that nothing executed and pages that implied they ran.
//!
//! Node launch and stop live here too, rather than being restated per frontend.
//! The CLI keeps its own thin wrapper because it must hand the process over on
//! exit; the browser and this loop share one code path and one supervisor.
//!
//! The engine is split by responsibility so each behaviour can be read on its
//! own: `state` owns what the loop is given and what it remembers, `engine`
//! owns the thread, and the remaining modules own one behaviour each.

mod alerts;
mod engine;
mod external;
mod launch;
mod probes;
mod restarts;
mod startup;
mod state;
mod upgrade;

pub use engine::Engine;
pub use launch::{launch_node, stop_node};
pub use state::EngineState;

#[cfg(test)]
#[path = "../../tests/unit/supervision/tests.rs"]
mod tests;
