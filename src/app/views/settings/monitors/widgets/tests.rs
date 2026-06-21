use super::status_label;

#[test]
fn monitor_status_label_matches_policy_enabled_state() {
    assert_eq!(status_label(true), "enabled");
    assert_eq!(status_label(false), "disabled");
}
