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
}

impl Drop for ProcessSupervisor {
    fn drop(&mut self) {
        for (_id, mut managed) in self.children.drain() {
            managed.terminate_on_drop();
        }
    }
}
