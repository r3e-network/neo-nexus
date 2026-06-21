use super::*;

#[test]
fn policy_summary_message_names_stopped_and_running_rollout_work() {
    let summary = RuntimeUpgradePolicySummary::new(
        2,
        RuntimeUpgradePolicyBreakdown {
            stopped_ready: 1,
            running_ready: 2,
            planned_stopped: 1,
            planned_running: 1,
            blocked_active: 1,
            current_or_unavailable: 1,
        },
        "Policy catalog".to_string(),
        true,
    );

    assert_eq!(
        summary.message(RuntimeUpgradeRunMode::Manual),
        "Runtime upgrade policy manual run via Policy catalog: 2 upgraded, 3 ready (1 stopped, 2 running), planned 2 (1 stopped, 1 running), 1 blocked, 1 current/unavailable; batch limited"
    );
}
