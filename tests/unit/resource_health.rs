use super::*;
use crate::{
    events::{EventKind, EventSeverity},
    repository::Repository,
};
use collector::{classify_disk, filesystem_space, memory_reading, Space};

fn report(available: u64, at: u64) -> ResourceReport {
    let policy = ResourcePolicy::default();
    ResourceReport {
        checked_at_unix: at,
        readings: vec![memory_reading(100, available, &policy)],
        policy,
        sample_failed: false,
    }
}

#[test]
fn thresholds_use_available_memory_and_include_quota_inode_readonly_failures() {
    let policy = ResourcePolicy::default();
    assert_eq!(
        memory_reading(100, 10, &policy).status,
        ResourceStatus::Warning
    );
    assert_eq!(
        memory_reading(100, 5, &policy).status,
        ResourceStatus::Critical
    );
    assert_eq!(
        memory_reading(0, 0, &policy).status,
        ResourceStatus::Unknown
    );
    assert_eq!(
        memory_reading(100, 101, &policy).status,
        ResourceStatus::Unknown
    );
    let mut space = Space {
        total: 100_000_000_000,
        available: 10_000_000_000,
        available_inodes: None,
        read_only: false,
    };
    assert_eq!(classify_disk(&space, &policy), ResourceStatus::Healthy);
    space.available_inodes = Some(0);
    assert_eq!(classify_disk(&space, &policy), ResourceStatus::Critical);
    space.available_inodes = None;
    space.read_only = true;
    assert_eq!(classify_disk(&space, &policy), ResourceStatus::Critical);
}

#[test]
fn warnings_and_recovery_are_confirmed_and_do_not_repeat_after_repository_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("resource.db");
    let repository = Repository::open(&db).unwrap();
    repository.record_resource_report(report(30, 100)).unwrap();
    repository.record_resource_report(report(9, 130)).unwrap();
    assert!(repository.list_recent_events(20).unwrap().is_empty());
    repository.record_resource_report(report(9, 160)).unwrap();
    let repository = Repository::open(db).unwrap();
    repository.record_resource_report(report(9, 190)).unwrap();
    repository.record_resource_report(report(30, 220)).unwrap();
    assert_eq!(repository.list_recent_events(20).unwrap().len(), 1);
    repository.record_resource_report(report(30, 250)).unwrap();
    let events = repository.list_recent_events(20).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind, EventKind::HostResourcesChanged);
    assert_eq!(events[0].severity, EventSeverity::Info);
    assert_eq!(events[1].severity, EventSeverity::Warning);
}

#[test]
fn critical_alerts_immediately_unknown_is_visible_and_stale_metrics_do_not_claim_zero() {
    let mut critical = report(1, 100);
    assert_eq!(
        settle(&mut critical, None)[0].severity,
        EventSeverity::Critical
    );
    assert!(settle(&mut report(1, 130), Some(&critical)).is_empty());
    let mut unavailable =
        collector::unavailable(std::path::Path::new("."), ResourcePolicy::default());
    assert!(!settle(&mut unavailable, None).is_empty());
    let text = prometheus(Some(&critical), true, 1000);
    assert!(text.contains("neonexus_resource_sample_fresh 0"));
    assert!(!text.contains("neonexus_resource_available_bytes{"));
}

#[test]
fn timeout_deduplicates_paths_and_clock_corrections_do_not_freeze_samples() {
    let dir = tempfile::tempdir().unwrap();
    let policy = ResourcePolicy {
        storage_paths: vec![dir.path().into(), dir.path().into()],
        ..ResourcePolicy::default()
    };
    let mut timeout = collector::unavailable(dir.path(), policy);
    assert_eq!(timeout.readings.len(), 2);
    assert_eq!(settle(&mut timeout, None).len(), 2);
    let repository = Repository::open(dir.path().join("clock.db")).unwrap();
    let now = collector::now();
    repository
        .record_resource_report(report(1, now + 3600))
        .unwrap();
    repository.record_resource_report(report(1, now)).unwrap();
    assert_eq!(
        repository
            .latest_resource_report()
            .unwrap()
            .unwrap()
            .checked_at_unix,
        now
    );
    assert_eq!(repository.list_recent_events(20).unwrap().len(), 1);
    // A merely delayed observation remains ignored after correction.
    repository
        .record_resource_report(report(50, now - 30))
        .unwrap();
    assert_eq!(
        repository
            .latest_resource_report()
            .unwrap()
            .unwrap()
            .checked_at_unix,
        now
    );
}

#[test]
fn policy_changes_reject_inflight_observations_and_backups_exclude_samples() {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("resource.db")).unwrap();
    let mut policy = ResourcePolicy {
        disk_warning_mib: 9000,
        ..ResourcePolicy::default()
    };
    repository.save_resource_policy(&policy).unwrap();
    repository.record_resource_report(report(1, 100)).unwrap();
    assert!(repository.latest_resource_report().unwrap().is_none());
    let backup = repository.list_workspace_settings_for_backup().unwrap();
    assert!(backup
        .iter()
        .any(|setting| setting.key == "resource_health.policy"));
    assert!(!backup
        .iter()
        .any(|setting| setting.key == "resource_health.snapshot"));
    policy.memory_warning_percent = policy.memory_critical_percent;
    assert!(repository.save_resource_policy(&policy).is_err());
}

#[test]
fn real_workspace_storage_can_be_sampled_without_writes() {
    let dir = tempfile::tempdir().unwrap();
    let disk = filesystem_space(dir.path()).unwrap();
    assert!(disk.total > 0);
    assert!(disk.available <= disk.total);
    assert!(filesystem_space(&dir.path().join("nonexistent-volume-dir")).is_err());
    let sample = collector::collect(dir.path(), ResourcePolicy::default());
    assert_eq!(sample.readings.len(), 2);
    assert!(sample
        .readings
        .iter()
        .all(|reading| reading.capacity_bytes.is_some()));
}

#[test]
fn a_failed_sample_write_does_not_leave_an_event_without_its_observation() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("resource.db");
    let repository = Repository::open(&db).unwrap();
    rusqlite::Connection::open(&db).unwrap().execute_batch("CREATE TRIGGER reject_resource BEFORE INSERT ON workspace_settings WHEN NEW.key='resource_health.snapshot' BEGIN SELECT RAISE(ABORT,'test sample failure'); END;").unwrap();
    assert!(repository.record_resource_report(report(1, 100)).is_err());
    assert!(repository.list_recent_events(20).unwrap().is_empty());
    assert!(repository.latest_resource_report().unwrap().is_none());
}
