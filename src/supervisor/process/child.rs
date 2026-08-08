use std::{
    path::PathBuf,
    process::{Child, ExitStatus},
};

use anyhow::{Context, Result};

use super::super::{ProcessExit, ProcessStart};

pub(super) struct ManagedChild {
    child: Child,
    log_path: PathBuf,
    /// The id this child was spawned for, carried so an exit is reported
    /// against it rather than against the reap loop's process key.
    ///
    /// Today `ManagedProcessSpec::id` is the node id for every managed process,
    /// so this equals the process key — the point is that it is read from the
    /// spec instead of being assumed equal to it, which is what a sidecar with
    /// its own key would need.
    node_id: String,
}

impl ManagedChild {
    pub(super) fn new(child: Child, log_path: PathBuf, node_id: String) -> Self {
        Self {
            child,
            log_path,
            node_id,
        }
    }

    pub(super) fn pid(&self) -> u32 {
        self.child.id()
    }

    pub(super) fn log_path(&self) -> &PathBuf {
        &self.log_path
    }

    pub(super) fn try_wait(&mut self, process_id: &str) -> Result<Option<ExitStatus>> {
        self.child
            .try_wait()
            .with_context(|| format!("failed to inspect process for {process_id}"))
    }

    pub(super) fn is_running(&mut self, label: &str) -> Result<bool> {
        self.child
            .try_wait()
            .with_context(|| format!("failed to inspect {label}"))
            .map(|status| status.is_none())
    }

    pub(super) fn to_start(&self) -> ProcessStart {
        ProcessStart {
            pid: self.pid(),
            log_path: self.log_path.clone(),
        }
    }

    pub(super) fn to_exit(&self, process_id: &str, status: ExitStatus) -> ProcessExit {
        ProcessExit {
            process_id: process_id.to_string(),
            node_id: self.node_id.clone(),
            pid: self.pid(),
            exit_code: status.code(),
        }
    }

    pub(super) fn child_mut(&mut self) -> &mut Child {
        &mut self.child
    }

    pub(super) fn terminate_on_drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
