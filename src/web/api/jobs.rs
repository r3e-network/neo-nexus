//! The background job list, as JSON.
//!
//! A long install is only observable if there is a way to ask what it is
//! doing. The pages render the same jobs server-side, but a script — a
//! deployment pipeline waiting for a runtime to land — needs the list in a
//! form it can poll.

use serde::Serialize;

use crate::web::jobs::Job;

/// One job, as a caller sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JobPayload {
    pub id: String,
    /// What serialises this job against others, e.g. `"runtime"`.
    pub lane: String,
    pub description: String,
    /// `running`, `done` or `failed`.
    pub status: String,
    /// The result text: what was produced, or why it failed.
    pub detail: String,
    /// Present while the job is running.
    pub progress: Option<JobProgressPayload>,
    pub started_at_unix: u64,
    pub finished_at_unix: Option<u64>,
}

/// Where a running job has got to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JobProgressPayload {
    /// The step being executed, in the operator's words.
    pub stage: String,
    /// Units finished, when the work can be counted.
    pub completed: Option<u64>,
    /// Units in total, when the work can be counted.
    pub total: Option<u64>,
    /// How far along the work is, when both counts are known.
    pub percent: Option<u8>,
    /// A one-line rendering of the above.
    pub summary: String,
}

/// The whole list, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JobListPayload {
    pub jobs: Vec<JobPayload>,
}

impl JobListPayload {
    pub fn from_jobs(jobs: &[Job]) -> Self {
        Self {
            jobs: jobs.iter().map(JobPayload::from).collect(),
        }
    }
}

impl From<&Job> for JobPayload {
    fn from(job: &Job) -> Self {
        Self {
            id: job.id.clone(),
            lane: job.lane.to_string(),
            description: job.description.clone(),
            status: job.status.label().to_string(),
            detail: job.detail.clone(),
            progress: job.progress.as_ref().map(JobProgressPayload::from),
            started_at_unix: job.started_at_unix,
            finished_at_unix: job.finished_at_unix,
        }
    }
}

impl From<&crate::web::jobs::JobProgress> for JobProgressPayload {
    fn from(progress: &crate::web::jobs::JobProgress) -> Self {
        Self {
            stage: progress.stage.clone(),
            completed: progress.completed,
            total: progress.total,
            percent: progress.percent(),
            summary: progress.summary(),
        }
    }
}
