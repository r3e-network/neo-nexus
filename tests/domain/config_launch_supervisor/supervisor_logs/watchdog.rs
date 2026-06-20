use crate::*;

#[test]
fn watchdog_schedules_exponential_restarts_until_exhausted() {
    let policy = RestartPolicy::new(3, Duration::from_secs(1), Duration::from_secs(5));
    let mut watchdog = Watchdog::new(policy);
    let now = Instant::now();

    assert_eq!(
        watchdog.record_failure("node-a", now),
        RestartOutcome::Scheduled {
            attempt: 1,
            delay: Duration::from_secs(1),
        }
    );
    assert_eq!(
        watchdog.status("node-a", now),
        WatchdogStatus::Pending {
            attempt: 1,
            remaining: Duration::from_secs(1),
        }
    );
    assert!(watchdog
        .due_restarts(now + Duration::from_millis(999))
        .is_empty());

    let due = watchdog.due_restarts(now + Duration::from_secs(1));
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].node_id, "node-a");
    assert_eq!(due[0].attempt, 1);
    assert_eq!(
        watchdog.status("node-a", now + Duration::from_secs(1)),
        WatchdogStatus::Idle
    );

    assert_eq!(
        watchdog.record_failure("node-a", now + Duration::from_secs(2)),
        RestartOutcome::Scheduled {
            attempt: 2,
            delay: Duration::from_secs(2),
        }
    );
    assert_eq!(
        watchdog.record_failure("node-a", now + Duration::from_secs(3)),
        RestartOutcome::Scheduled {
            attempt: 3,
            delay: Duration::from_secs(4),
        }
    );
    assert_eq!(
        watchdog.record_failure("node-a", now + Duration::from_secs(4)),
        RestartOutcome::Exhausted { attempts: 3 }
    );
    assert_eq!(
        watchdog.status("node-a", now + Duration::from_secs(4)),
        WatchdogStatus::Exhausted { attempts: 3 }
    );
}

#[test]
fn watchdog_caps_delay_and_clears_state() {
    let policy = RestartPolicy::new(4, Duration::from_secs(3), Duration::from_secs(5));
    let mut watchdog = Watchdog::new(policy);
    let now = Instant::now();

    assert_eq!(
        watchdog.record_failure("node-a", now),
        RestartOutcome::Scheduled {
            attempt: 1,
            delay: Duration::from_secs(3),
        }
    );
    assert_eq!(
        watchdog.record_failure("node-a", now),
        RestartOutcome::Scheduled {
            attempt: 2,
            delay: Duration::from_secs(5),
        }
    );
    assert!(watchdog.has_pending_restart());

    watchdog.clear("node-a");

    assert_eq!(watchdog.status("node-a", now), WatchdogStatus::Idle);
    assert!(!watchdog.has_pending_restart());
}

#[test]
fn watchdog_policy_can_disable_restarts_and_clear_pending_state() {
    let now = Instant::now();
    let mut watchdog = Watchdog::new(RestartPolicy::new(
        3,
        Duration::from_secs(1),
        Duration::from_secs(10),
    ));

    assert_eq!(
        watchdog.record_failure("node-a", now),
        RestartOutcome::Scheduled {
            attempt: 1,
            delay: Duration::from_secs(1),
        }
    );
    assert!(watchdog.has_pending_restart());

    watchdog.update_policy(RestartPolicy::with_enabled(
        false,
        3,
        Duration::from_secs(1),
        Duration::from_secs(10),
    ));

    assert!(!watchdog.has_pending_restart());
    assert_eq!(
        watchdog.record_failure("node-a", now),
        RestartOutcome::Disabled
    );
    assert_eq!(watchdog.status("node-a", now), WatchdogStatus::Idle);
}
