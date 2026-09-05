use super::{SupervisionHeartbeat, SupervisionLiveness};

#[test]
fn an_engine_that_never_ran_is_reported_not_running() {
    let heartbeat = SupervisionHeartbeat::new();
    assert_eq!(
        heartbeat.evaluate_at(1_000).liveness,
        SupervisionLiveness::NotRunning
    );
}

#[test]
fn a_beating_loop_is_running() {
    let heartbeat = SupervisionHeartbeat::new();
    heartbeat.beat_at(1);
    assert_eq!(
        heartbeat.evaluate_at(1).liveness,
        SupervisionLiveness::Running
    );
}

/// The whole point of the heartbeat: ticks that stop completing age into a
/// stalled verdict instead of a green light over an unsupervised fleet.
#[test]
fn a_loop_that_stops_completing_ticks_is_stalled() {
    let heartbeat = SupervisionHeartbeat::new();
    heartbeat.beat_at(1);
    // 90 seconds without a completed tick: past the stall bound.
    assert_eq!(
        heartbeat.evaluate_at(1 + 91).liveness,
        SupervisionLiveness::Stalled
    );
    // Just inside it, the loop is still trusted.
    assert_eq!(
        heartbeat.evaluate_at(1 + 89).liveness,
        SupervisionLiveness::Running
    );
}

#[test]
fn a_failed_spawn_is_failed_even_with_heartbeats() {
    let heartbeat = SupervisionHeartbeat::new();
    heartbeat.beat_at(1);
    heartbeat.mark_failed("supervision thread did not start".to_string());
    let report = heartbeat.evaluate_at(1);
    assert_eq!(report.liveness, SupervisionLiveness::Failed);
    assert_eq!(
        report.detail.as_deref(),
        Some("supervision thread did not start")
    );
}
