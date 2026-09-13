use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
    time::{Duration, Instant},
};

use anyhow::Result;

use crate::{launch::LaunchPlan, types::NodeConfig};

use super::{reap::reap_finished_children, spawn::spawn_managed_child, ProcessSupervisor};
use crate::supervisor::{
    termination::{stop_by_pid, stop_child},
    LaunchConfirmation, ManagedProcessSpec, PidStop, ProcessExit, ProcessStart, ProcessStop,
};

/// How long a freshly spawned process is watched before its launch counts as
/// successful.
///
/// This is not a readiness or health window — a Neo node takes minutes to sync
/// and must not be waited on here. It is only long enough to catch a process
/// that dies of its own arguments: a rejected flag, an unparseable config, a
/// port already bound. Those all fail within a few tens of milliseconds, and
/// the cost of looking is one barely perceptible pause on the launch button.
pub const LAUNCH_SETTLE_WINDOW: Duration = Duration::from_millis(600);

/// How often the window is sampled. `try_wait` is a `waitpid(WNOHANG)`, so this
/// is cheap enough to ask often and still return the instant a child dies.
const LAUNCH_SETTLE_POLL: Duration = Duration::from_millis(25);

/// How much of the child's own output a failure report quotes.
const STARTUP_OUTPUT_BUDGET: usize = 8 * 1024;

/// How many of its last lines are worth showing.
const STARTUP_OUTPUT_LINES: usize = 6;

impl ProcessSupervisor {
    /// Watch a just-started process for [`LAUNCH_SETTLE_WINDOW`] and report
    /// whether it survived its own startup.
    ///
    /// Returns as soon as the child exits, so a healthy launch pays the full
    /// window and a broken one is reported immediately.
    ///
    /// A process this supervisor holds no handle for — one started by another
    /// process, or already reaped — cannot be judged here and is reported as
    /// surviving. Inventing a failure for a node we cannot see would be worse
    /// than the silence it replaces.
    pub fn confirm_startup(&mut self, process_id: &str, window: Duration) -> LaunchConfirmation {
        let deadline = Instant::now() + window;
        loop {
            let Some(managed) = self.children.get_mut(process_id) else {
                return LaunchConfirmation::Survived;
            };
            match managed.try_wait(process_id) {
                Ok(Some(status)) => {
                    let output = read_startup_output(managed.log_path(), managed.output_offset());
                    self.children.remove(process_id);
                    return LaunchConfirmation::ExitedDuringStartup {
                        exit_code: status.code(),
                        output,
                    };
                }
                // Still running, or the handle cannot be inspected. Neither is
                // evidence of failure.
                Ok(None) => {}
                Err(_) => return LaunchConfirmation::Survived,
            }
            if Instant::now() >= deadline {
                return LaunchConfirmation::Survived;
            }
            std::thread::sleep(LAUNCH_SETTLE_POLL);
        }
    }

    pub fn start(
        &mut self,
        node: &NodeConfig,
        plan: &LaunchPlan,
        log_path: impl AsRef<Path>,
    ) -> Result<ProcessStart> {
        ensure_node_runtime_bound(node)?;
        let spec = ManagedProcessSpec::for_node(node, plan);
        self.start_process(&spec, log_path)
    }

    pub fn start_process(
        &mut self,
        spec: &ManagedProcessSpec,
        log_path: impl AsRef<Path>,
    ) -> Result<ProcessStart> {
        if let Some(start) = self.reuse_running_child(spec)? {
            return Ok(start);
        }

        let log_path = log_path.as_ref().to_path_buf();
        let (managed, start) = spawn_managed_child(spec, log_path)?;
        self.children.insert(spec.id.clone(), managed);
        // A managed (re)launch invalidates any observation carried over from a
        // previous generation of this node, including PID reuse: the collector
        // keeps its byte offset but must not republish a stale sync sample from
        // before this launch. The epoch bump gates that.
        self.observations().invalidate(&spec.id);
        Ok(start)
    }

    pub fn stop(&mut self, node_id: &str) -> Result<Option<ProcessStop>> {
        self.stop_process(node_id)
    }

    pub fn stop_process(&mut self, process_id: &str) -> Result<Option<ProcessStop>> {
        if let Some(mut managed) = self.children.remove(process_id) {
            let log_path = managed.log_path().clone();
            let stop = stop_child(
                process_id,
                managed.child_mut(),
                log_path,
                self.stop_grace_period,
            );
            return match stop {
                Ok(stop) => Ok(Some(stop)),
                Err(error) => {
                    // A failed termination is not ownership transfer. Retain
                    // the handle so a retry, reaper, or Drop can still reach
                    // the process instead of silently orphaning it.
                    self.children.insert(process_id.to_string(), managed);
                    Err(error)
                }
            };
        }
        Ok(None)
    }

    /// Stop a node this supervisor never spawned, from the pid the workspace
    /// recorded for it.
    ///
    /// [`Self::stop_process`] correctly reports `None` when there is no handle,
    /// and `None` there means "not mine to stop" — not "stopped". A server
    /// restart or a `--node-start` from another process leaves exactly this
    /// state: a node running with no `Child` anywhere here.
    pub fn stop_recorded_pid(&self, node: &NodeConfig, log_path: impl AsRef<Path>) -> PidStop {
        stop_by_pid(
            node,
            log_path.as_ref().to_path_buf(),
            self.stop_grace_period,
        )
    }

    pub fn restart(
        &mut self,
        node: &NodeConfig,
        plan: &LaunchPlan,
        log_path: impl AsRef<Path>,
    ) -> Result<ProcessStart> {
        ensure_node_runtime_bound(node)?;
        let spec = ManagedProcessSpec::for_node(node, plan);
        self.restart_process(&spec, log_path)
    }

    pub fn restart_process(
        &mut self,
        spec: &ManagedProcessSpec,
        log_path: impl AsRef<Path>,
    ) -> Result<ProcessStart> {
        let _ = self.stop_process(&spec.id)?;
        self.start_process(spec, log_path)
    }

    pub fn reap_finished(&mut self) -> Result<Vec<ProcessExit>> {
        reap_finished_children(&mut self.children)
    }

    fn reuse_running_child(&mut self, spec: &ManagedProcessSpec) -> Result<Option<ProcessStart>> {
        if let Some(managed) = self.children.get_mut(&spec.id) {
            if managed.is_running(&spec.label)? {
                return Ok(Some(managed.to_start()));
            }
            self.children.remove(&spec.id);
        }
        Ok(None)
    }
}

/// Read what the child itself wrote, starting past NeoNexus's launch header.
///
/// Best effort by design: this runs while reporting a failure that has already
/// happened, so an unreadable log degrades to an empty quote rather than
/// replacing the real reason with an IO error.
fn read_startup_output(log_path: &Path, offset: u64) -> String {
    let Ok(mut file) = std::fs::File::open(log_path) else {
        return String::new();
    };
    if file.seek(SeekFrom::Start(offset)).is_err() {
        return String::new();
    }
    let mut buffer = Vec::new();
    if file
        .take(STARTUP_OUTPUT_BUDGET as u64)
        .read_to_end(&mut buffer)
        .is_err()
    {
        return String::new();
    }
    let text = String::from_utf8_lossy(&buffer);
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .collect();
    let tail = lines.len().saturating_sub(STARTUP_OUTPUT_LINES);
    lines[tail..].join("; ")
}

fn ensure_node_runtime_bound(node: &NodeConfig) -> Result<()> {
    if node.binary_path.as_os_str().is_empty() {
        anyhow::bail!(
            "node {} has no trusted local runtime; rebind it before launch",
            node.name
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/supervisor/lifecycle/tests.rs"]
mod tests;
