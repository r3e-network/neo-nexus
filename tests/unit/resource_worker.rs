use super::*;

#[test]
fn a_stuck_storage_call_alerts_without_blocking_or_creating_more_workers() {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("worker.db")).unwrap();
    let (sender, receiver) = mpsc::channel();
    let policy = ResourcePolicy::default();
    let mut monitor = ResourceMonitor {
        active: Some(Sampling {
            receiver,
            started: Instant::now() - Duration::from_secs(11),
            policy: policy.clone(),
            timed_out: false,
        }),
        last_sample: Some(Instant::now()),
        last_policy: Some(policy.clone()),
        reported_error: false,
    };
    monitor.poll(&repository).unwrap();
    assert!(monitor.active.as_ref().unwrap().timed_out);
    assert!(
        repository
            .latest_resource_report()
            .unwrap()
            .unwrap()
            .sample_failed
    );
    let events = repository.list_recent_events(20).unwrap().len();
    monitor.poll(&repository).unwrap();
    assert!(monitor.active.is_some());
    assert_eq!(repository.list_recent_events(20).unwrap().len(), events);
    // A late successful result is discarded, then the next interval may sample.
    sender
        .send(super::super::collector::collect(dir.path(), policy))
        .unwrap();
    monitor.poll(&repository).unwrap();
    assert!(monitor.active.is_none());
    assert!(
        repository
            .latest_resource_report()
            .unwrap()
            .unwrap()
            .sample_failed
    );
}
