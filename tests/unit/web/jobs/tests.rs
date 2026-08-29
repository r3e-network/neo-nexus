//! The job registry's promises: one job per lane at a time, a result that
//! outlives the request that asked for it, and bounded history.

use std::sync::mpsc;
use std::time::Duration;

use super::*;

/// Wait for a predicate over the registry rather than sleeping a guessed amount.
fn wait_for(jobs: &Jobs, wanted: impl Fn(&[Job]) -> bool) -> bool {
    for _ in 0..100 {
        if wanted(&jobs.recent()) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn a_submitted_job_reports_its_result_when_it_finishes() {
    let jobs = Jobs::default();
    let job = jobs
        .submit("test", "record a value".to_string(), || {
            Ok("value=7".to_string())
        })
        .expect("lane is free");
    assert_eq!(job.status, JobStatus::Running);

    let settled = wait_for(&jobs, |seen| {
        seen.iter()
            .any(|seen| seen.id == job.id && seen.status == JobStatus::Succeeded)
    });
    assert!(settled, "job never reported success");
    let finished = jobs
        .recent()
        .into_iter()
        .find(|seen| seen.id == job.id)
        .expect("recorded");
    assert_eq!(finished.detail, "value=7");
    assert!(finished.finished_at_unix.is_some());
}

#[test]
fn a_failing_job_records_the_reason_and_stays_visible() {
    let jobs = Jobs::default();
    let job = jobs
        .submit("test", "will fail".to_string(), || {
            Err("disk full".to_string())
        })
        .expect("lane is free");
    assert!(wait_for(&jobs, |seen| {
        seen.iter()
            .any(|seen| seen.id == job.id && seen.status == JobStatus::Failed)
    }));
    let failed = jobs
        .recent()
        .into_iter()
        .find(|seen| seen.id == job.id)
        .expect("recorded");
    assert_eq!(failed.detail, "disk full");
}

#[test]
fn one_lane_runs_one_job_at_a_time() {
    let (began_tx, began_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let jobs = Jobs::default();
    let first = jobs
        .submit("runtime", "download neo-go".to_string(), move || {
            let _ = began_tx.send(());
            // Hold the lane until the test says so.
            let _ = release_rx.recv();
            Ok("done".to_string())
        })
        .expect("lane starts free");

    began_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("worker began");
    assert!(jobs.is_busy("runtime"));

    let refused = jobs.submit("runtime", "install too".to_string(), || Ok(String::new()));
    match refused {
        Err(busy) => assert_eq!(busy.description, "download neo-go"),
        Ok(job) => unreachable!("a second runtime job was accepted: {}", job.id),
    }

    // A different lane is unaffected.
    assert!(jobs
        .submit("other", "unrelated".to_string(), || Ok("ok".to_string()))
        .is_ok());

    release_tx.send(()).expect("release worker");
    assert!(wait_for(&jobs, |seen| {
        seen.iter()
            .any(|seen| seen.id == first.id && seen.status == JobStatus::Succeeded)
    }));
    // Once the lane is free again the same request is accepted.
    assert!(!jobs.is_busy("runtime"));
    assert!(jobs
        .submit(
            "runtime",
            "install now".to_string(),
            || Ok("ok".to_string())
        )
        .is_ok());
}

#[test]
fn history_stays_bounded_and_newest_first() {
    let jobs = Jobs::default();
    for index in 0..(HISTORY + 10) {
        let _ = jobs.submit("test", format!("job {index}"), || Ok("done".to_string()));
        // Each lane run must finish before the next is accepted.
        let seen = jobs.clone();
        wait_for(&seen, |jobs| jobs.iter().all(|job| !job.status.is_open()));
    }
    let recent = jobs.recent();
    assert!(recent.len() <= HISTORY, "history grew to {}", recent.len());
    let numbers = recent
        .iter()
        .filter_map(|job| job.description.rsplit(' ').next()?.parse::<usize>().ok())
        .collect::<Vec<_>>();
    let mut descending = numbers.clone();
    descending.sort_unstable();
    descending.reverse();
    assert_eq!(numbers, descending, "recent() must be newest first");
}

#[test]
fn status_labels_are_stable_enough_to_render_against() {
    assert_eq!(JobStatus::Running.label(), "running");
    assert_eq!(JobStatus::Succeeded.label(), "done");
    assert_eq!(JobStatus::Failed.label(), "failed");
    assert!(JobStatus::Running.is_open());
    assert!(!JobStatus::Succeeded.is_open());
}
