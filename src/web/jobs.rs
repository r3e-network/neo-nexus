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
    pub started_at_unix: u64,
    pub finished_at_unix: Option<u64>,
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

impl Jobs {
    /// Run `work` on its own thread. The job is recorded as running before this
    /// returns, so a page that reloads immediately still sees it.
    pub fn submit<F>(&self, lane: &'static str, description: String, work: F) -> Result<Job, Busy>
    where
        F: FnOnce() -> Result<String, String> + Send + 'static,
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
            let (status, detail) = match work() {
                Ok(detail) => (JobStatus::Succeeded, detail),
                Err(detail) => (JobStatus::Failed, detail),
            };
            let mut registry = shared
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(job) = registry.jobs.iter_mut().find(|job| job.id == finished.id) {
                job.status = status;
                job.detail = detail;
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
