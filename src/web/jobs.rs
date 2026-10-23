//! Background work with a visible outcome.
//!
//! Fetching a runtime package can take minutes. Doing it inside a request would
//! hold a connection open, time out the browser, and leave the operator with
//! nothing to look at — and a page reload would look like the work had been
//! abandoned. So the page submits a job, the work runs on its own thread, and
//! the page polls a small registry for the result.
//!
//! Jobs are grouped into lanes, and a lane runs one job at a time. Two
//! concurrent installs into the same root could interleave writes and leave a
//! half-populated directory behind, which is worse than making the second
//! operator wait.

use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

/// How much history a page shows. Enough to see what just happened, small
/// enough that a long-lived server does not grow without bound.
const HISTORY: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Succeeded,
    Failed,
}

impl JobStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Succeeded => "done",
            Self::Failed => "failed",
        }
    }

    pub fn is_open(&self) -> bool {
        matches!(self, Self::Running)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: String,
    /// What serialises this job against others, e.g. `"runtime"`.
    pub lane: &'static str,
    pub description: String,
    pub status: JobStatus,
    /// The result text: what was produced, or why it failed.
    pub detail: String,
    /// What the job is doing right now, while it is still running.
    pub progress: Option<JobProgress>,
    pub started_at_unix: u64,
    pub finished_at_unix: Option<u64>,
}

/// Where a running job has got to.
///
/// A job that takes minutes and reports only "running" is indistinguishable
/// from one that has hung. The stage says which step is executing, and the
/// counts say how far along it is whenever the work can be counted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobProgress {
    /// The step being executed, in the operator's words.
    pub stage: String,
    /// Units finished, when the work can be counted.
    pub completed: Option<u64>,
    /// Units in total, when the work can be counted.
    pub total: Option<u64>,
}

impl JobProgress {
    /// How far along the work is, when both counts are known.
    pub fn percent(&self) -> Option<u8> {
        let total = self.total?;
        if total == 0 {
            return Some(100);
        }
        let completed = self.completed.unwrap_or(0).min(total);
        let percent = completed.saturating_mul(100) / total;
        Some(u8::try_from(percent).unwrap_or(100))
    }

    /// A one-line description for a page or an API caller.
    pub fn summary(&self) -> String {
        match (self.completed, self.total, self.percent()) {
            (Some(completed), Some(total), Some(percent)) => {
                format!(
                    "{stage} ({completed}/{total}, {percent}%)",
                    stage = self.stage
                )
            }
            (Some(completed), None, _) => {
                format!("{stage} ({completed} done)", stage = self.stage)
            }
            _ => self.stage.clone(),
        }
    }
}

#[derive(Default)]
struct Registry {
    jobs: Vec<Job>,
}

/// Shared handle to the job list. Cloning is cheap and every clone sees the
/// same work, which is what lets a handler, a page, and a worker thread agree.
#[derive(Clone, Default)]
pub struct Jobs(Arc<Mutex<Registry>>);

/// Why a job was not accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Busy {
    pub description: String,
}

/// The handle a running job uses to say what it is doing.
///
/// It is handed to the work closure and writes into the same registry the page
/// reads, so a refresh shows the current step rather than a blank "running".
/// A report that arrives after the job has already finished is ignored: the
/// finished outcome is the truth, and a late stage message must not reopen it.
pub struct JobReporter {
    registry: Arc<Mutex<Registry>>,
    id: String,
}

impl JobReporter {
    /// Report the step the work has reached.
    pub fn stage(&self, stage: impl Into<String>) {
        self.publish(JobProgress {
            stage: stage.into(),
            completed: None,
            total: None,
        });
    }

    /// Report counted progress, for work whose steps can be numbered.
    pub fn count(&self, completed: u64, total: Option<u64>, stage: impl Into<String>) {
        self.publish(JobProgress {
            stage: stage.into(),
            completed: Some(completed),
            total,
        });
    }

    fn publish(&self, progress: JobProgress) {
        let mut registry = self
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(job) = registry
            .jobs
            .iter_mut()
            .find(|job| job.id == self.id && job.status.is_open())
        {
            job.progress = Some(progress);
        }
    }
}

impl Jobs {
    /// Run `work` on its own thread. The job is recorded as running before this
    /// returns, so a page that reloads immediately still sees it.
    ///
    /// The work receives a reporter it can use to say which step it is on, so
    /// a long install shows what it is doing instead of only that it started.
    pub fn submit<F>(&self, lane: &'static str, description: String, work: F) -> Result<Job, Busy>
    where
        F: FnOnce(&JobReporter) -> Result<String, String> + Send + 'static,
    {
        let job = {
            let mut registry = self.lock();
            if let Some(running) = registry
                .jobs
                .iter()
                .find(|job| job.lane == lane && job.status.is_open())
            {
                return Err(Busy {
                    description: running.description.clone(),
                });
            }
            let job = Job {
                id: uuid::Uuid::new_v4().simple().to_string(),
                lane,
                description,
                status: JobStatus::Running,
                detail: String::new(),
                progress: None,
                started_at_unix: now_unix(),
                finished_at_unix: None,
            };
            registry.jobs.push(job.clone());
            registry.trim();
            job
        };

        let shared = Arc::clone(&self.0);
        let finished = job.clone();
        let name = format!("neonexus-job-{lane}");
        // A thread that fails to spawn is reported rather than swallowed: the
        // job is already in the list, and leaving it "running" forever would be
        // a lie worse than an immediate failure.
        match std::thread::Builder::new().name(name).spawn(move || {
            let reporter = JobReporter {
                registry: Arc::clone(&shared),
                id: finished.id.clone(),
            };
            let (status, detail) = match work(&reporter) {
                Ok(detail) => (JobStatus::Succeeded, detail),
                Err(detail) => (JobStatus::Failed, detail),
            };
            let mut registry = shared
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(job) = registry.jobs.iter_mut().find(|job| job.id == finished.id) {
                job.status = status;
                job.detail = detail;
                job.progress = None;
                job.finished_at_unix = Some(now_unix());
            }
        }) {
            Ok(_) => Ok(job),
            Err(error) => {
                self.fail(&job.id, format!("could not start the work: {error}"));
                Err(Busy {
                    description: job.description,
                })
            }
        }
    }

    /// A snapshot for rendering, newest first.
    pub fn recent(&self) -> Vec<Job> {
        let mut jobs = self.lock().jobs.clone();
        jobs.reverse();
        jobs
    }

    pub fn is_busy(&self, lane: &str) -> bool {
        self.lock()
            .jobs
            .iter()
            .any(|job| job.lane == lane && job.status.is_open())
    }

    fn fail(&self, id: &str, detail: String) {
        let mut registry = self.lock();
        if let Some(job) = registry.jobs.iter_mut().find(|job| job.id == id) {
            job.status = JobStatus::Failed;
            job.detail = detail;
            job.finished_at_unix = Some(now_unix());
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Registry> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Registry {
    fn trim(&mut self) {
        if self.jobs.len() > HISTORY {
            let excess = self.jobs.len() - HISTORY;
            self.jobs.drain(..excess);
        }
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "../../tests/unit/web/jobs/tests.rs"]
mod tests;
