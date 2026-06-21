use super::NodeStatus;

#[test]
fn node_status_operator_labels_are_title_case() {
    assert_eq!(NodeStatus::Running.label(), "Running");
    assert_eq!(NodeStatus::Starting.label(), "Starting");
    assert_eq!(NodeStatus::Stopped.label(), "Stopped");
    assert_eq!(NodeStatus::Error.label(), "Error");
}

#[test]
fn node_status_lifecycle_predicates_capture_operator_safety() {
    assert!(NodeStatus::Running.is_active());
    assert!(NodeStatus::Starting.is_active());
    assert!(!NodeStatus::Stopped.is_active());
    assert!(!NodeStatus::Error.is_active());
    assert!(NodeStatus::Running.is_running());
    assert!(!NodeStatus::Starting.is_running());
    assert!(NodeStatus::Starting.is_starting());
    assert!(!NodeStatus::Running.is_starting());
    assert!(NodeStatus::Stopped.is_stopped());
    assert!(!NodeStatus::Error.is_stopped());
}
