use std::collections::HashMap;

use anyhow::Result;

use super::child::ManagedChild;
use crate::supervisor::ProcessExit;

pub(super) fn reap_finished_children(
    children: &mut HashMap<String, ManagedChild>,
) -> Result<Vec<ProcessExit>> {
    let mut exits = Vec::new();
    let mut finished = Vec::new();

    for (process_id, managed) in children.iter_mut() {
        if let Some(status) = managed.try_wait(process_id)? {
            exits.push(managed.to_exit(process_id, status));
            finished.push(process_id.clone());
        }
    }

    for process_id in finished {
        children.remove(&process_id);
    }

    Ok(exits)
}
