use super::{
    collector::{collect, unavailable, workspace_path},
    ResourcePolicy, ResourceReport, ResourceStatus,
};
use crate::{
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    repository::Repository,
};
use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    time::{Duration, Instant},
};

const SAMPLE_TIMEOUT: Duration = Duration::from_secs(10);

struct Sampling {
    receiver: Receiver<ResourceReport>,
    started: Instant,
    policy: ResourcePolicy,
    timed_out: bool,
}

#[derive(Default)]
pub struct ResourceMonitor {
    active: Option<Sampling>,
    last_sample: Option<Instant>,
    last_policy: Option<ResourcePolicy>,
    reported_error: bool,
}

impl ResourceMonitor {
    pub fn tick(&mut self, repository: &Repository) {
        if let Err(error) = self.poll(repository) {
            if !self.reported_error {
                eprintln!("neo-nexus: resource monitoring failed: {error}");
                self.reported_error = true;
            }
        } else {
            self.reported_error = false;
        }
    }

    fn poll(&mut self, repository: &Repository) -> anyhow::Result<()> {
        let policy = repository.load_resource_policy()?;
        let workspace = workspace_path(repository);
        if let Some(sample) = &mut self.active {
            match sample.receiver.try_recv() {
                Ok(report) => {
                    let current = !sample.timed_out && sample.policy == policy && policy.enabled;
                    self.active = None;
                    if current {
                        repository.record_resource_report(report)?;
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    self.active = None;
                    if policy.enabled {
                        repository
                            .record_resource_report(unavailable(&workspace, policy.clone()))?;
                    }
                }
                Err(TryRecvError::Empty) => {
                    if sample.started.elapsed() >= SAMPLE_TIMEOUT && !sample.timed_out {
                        sample.timed_out = true;
                        if policy.enabled {
                            repository
                                .record_resource_report(unavailable(&workspace, policy.clone()))?;
                        }
                    }
                    // Keep exactly one outstanding worker. A blocked filesystem
                    // cannot create a new orphan thread on every guardian tick.
                    return Ok(());
                }
            }
        }
        if !policy.enabled {
            self.last_policy = Some(policy);
            return Ok(());
        }
        if self.last_policy.as_ref() == Some(&policy)
            && self
                .last_sample
                .is_some_and(|last| last.elapsed() < Duration::from_secs(policy.interval_seconds))
        {
            return Ok(());
        }
        let (sender, receiver) = mpsc::channel();
        let sampled_policy = policy.clone();
        std::thread::Builder::new()
            .name("neonexus-resources".into())
            .spawn(move || {
                let report = collect(&workspace, sampled_policy);
                let _ = sender.send(report);
            })?;
        let instant = Instant::now();
        self.active = Some(Sampling {
            receiver,
            started: instant,
            policy: policy.clone(),
            timed_out: false,
        });
        self.last_sample = Some(instant);
        self.last_policy = Some(policy);
        Ok(())
    }
}

/// Persist confirmation with the sample, so restarting the workbench cannot
/// emit an unchanged warning again or forget a pending recovery observation.
pub(crate) fn settle(
    report: &mut ResourceReport,
    previous: Option<&ResourceReport>,
) -> Vec<NewRuntimeEvent> {
    let previous = previous.filter(|prior| prior.policy == report.policy);
    let consecutive = previous.is_some_and(|prior| {
        prior.checked_at_unix < report.checked_at_unix && prior.is_fresh(report.checked_at_unix)
    });
    let mut events = vec![];
    for reading in &mut report.readings {
        let prior =
            previous.and_then(|report| report.readings.iter().find(|old| old.id == reading.id));
        reading.pending_samples = prior
            .filter(|old| consecutive && old.status == reading.status)
            .map_or(1, |old| old.pending_samples.saturating_add(1).min(2));
        if report.sample_failed {
            reading.pending_samples = 2;
        }
        let old_status = prior.map_or(ResourceStatus::Unknown, |old| old.confirmed_status);
        reading.confirmed_status =
            if reading.status == ResourceStatus::Critical || reading.pending_samples >= 2 {
                reading.status
            } else {
                old_status
            };
        // The first healthy sample establishes a baseline without an alert.
        if prior.is_none() && reading.status == ResourceStatus::Healthy {
            reading.confirmed_status = ResourceStatus::Healthy;
            reading.alerted = true;
            continue;
        }
        reading.alerted =
            prior.is_some_and(|old| old.alerted) && reading.confirmed_status == old_status;
        if reading.alerted
            || (reading.confirmed_status == ResourceStatus::Unknown && reading.pending_samples < 2)
        {
            continue;
        }
        events.push(NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::HostResourcesChanged,
            severity: match reading.confirmed_status {
                ResourceStatus::Healthy => EventSeverity::Info,
                ResourceStatus::Critical => EventSeverity::Critical,
                ResourceStatus::Warning | ResourceStatus::Unknown => EventSeverity::Warning,
            },
            message: format!(
                "Host resources {}: {}; {}",
                reading.label,
                reading.confirmed_status.label(),
                reading.message
            ),
        });
        reading.alerted = true;
    }
    events
}

#[cfg(test)]
#[path = "../../tests/unit/resource_worker.rs"]
mod tests;
