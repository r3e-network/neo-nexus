use std::{collections::HashMap, time::Duration};

use super::model::DEFAULT_STOP_GRACE_PERIOD;

mod child;
mod lifecycle;
mod reap;
mod spawn;

use child::ManagedChild;

pub struct ProcessSupervisor {
    children: HashMap<String, ManagedChild>,
    stop_grace_period: Duration,
}

impl Default for ProcessSupervisor {
    fn default() -> Self {
        Self {
            children: HashMap::new(),
            stop_grace_period: DEFAULT_STOP_GRACE_PERIOD,
        }
    }
}

impl ProcessSupervisor {
    pub fn with_stop_grace_period(stop_grace_period: Duration) -> Self {
        Self {
            children: HashMap::new(),
            stop_grace_period,
        }
    }

    /// Forget every child **without terminating it**.
    ///
    /// [`Drop`] kills what is still registered, which is what a desktop shell
    /// wants and what a one-shot `--node-start` must not do: without this, the
    /// CLI reports a node as started and then kills it on the way out of the
    /// process. Dropping the stored handle is not itself a signal — `ManagedChild`
    /// has no `Drop`, and dropping a `std::process::Child` leaves the OS process
    /// running.
    pub fn disown_all(&mut self) {
        self.children.clear();
    }

    /// Forget one child without terminating it, for the case where a single
    /// process is handed to someone else to supervise.
    pub fn disown(&mut self, process_id: &str) -> bool {
        self.children.remove(process_id).is_some()
    }

    /// Whether this supervisor can actually control `node_id` — i.e. holds the
    /// handle it was started with. A node can be alive in the database and
    /// unmanaged here, and reporting the two as the same thing is how a
    /// "stopped" node keeps running.
    pub fn is_managing(&self, node_id: &str) -> bool {
        self.children.contains_key(node_id)
    }

    /// Every node this supervisor can control.
    pub fn managed_node_ids(&self) -> Vec<String> {
        self.children.keys().cloned().collect()
    }
}

impl Drop for ProcessSupervisor {
    fn drop(&mut self) {
        for (_id, mut managed) in self.children.drain() {
            managed.terminate_on_drop();
        }
    }
}
