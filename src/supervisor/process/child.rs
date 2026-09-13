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
    /// Where this generation's own output starts in the shared log file, i.e.
    /// the length of the file once NeoNexus had finished writing its launch
    /// header. Reading from here yields what the child said and nothing this
    /// process wrote about it, which is what a failure report needs to quote.
    output_offset: u64,
    /// When this generation was spawned, in unix seconds.
    ///
    /// The health layer needs it to tell "this client is still opening its
    /// store" from "this node is not answering". A client that takes three
    /// minutes to come up has not failed; it has not been asked yet. Only
    /// processes this supervisor launched have one — a node adopted from a
    /// previous run reports no uptime rather than a guessed one.
    started_at_unix: u64,
}

impl ManagedChild {
    pub(super) fn new(
        child: Child,
        log_path: PathBuf,
        node_id: String,
        output_offset: u64,
        started_at_unix: u64,
    ) -> Self {
        Self {
            child,
            log_path,
            node_id,
            output_offset,
            started_at_unix,
        }
    }

    pub(super) fn started_at_unix(&self) -> u64 {
        self.started_at_unix
    }

    pub(super) fn pid(&self) -> u32 {
        self.child.id()
    }

    pub(super) fn log_path(&self) -> &PathBuf {
        &self.log_path
    }

    pub(super) fn output_offset(&self) -> u64 {
        self.output_offset
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
