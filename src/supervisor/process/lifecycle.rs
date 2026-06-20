use std::path::Path;

use anyhow::Result;

use crate::{launch::LaunchPlan, types::NodeConfig};

use super::{reap::reap_finished_children, spawn::spawn_managed_child, ProcessSupervisor};
use crate::supervisor::{
    termination::stop_child, ManagedProcessSpec, ProcessExit, ProcessStart, ProcessStop,
};

impl ProcessSupervisor {
    pub fn start(
        &mut self,
        node: &NodeConfig,
        plan: &LaunchPlan,
        log_path: impl AsRef<Path>,
    ) -> Result<ProcessStart> {
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
            )?;
            return Ok(Some(stop));
        }
        Ok(None)
    }

    pub fn restart(
        &mut self,
        node: &NodeConfig,
        plan: &LaunchPlan,
        log_path: impl AsRef<Path>,
    ) -> Result<ProcessStart> {
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
