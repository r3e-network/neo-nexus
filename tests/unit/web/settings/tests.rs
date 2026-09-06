use std::{collections::HashSet, time::Duration};

use super::*;

#[test]
fn settings_forms_assign_document_unique_control_ids() {
    let watchdog = RestartPolicy::new(3, Duration::from_secs(2), Duration::from_secs(30));
    let markup = format!(
        "{}{}{}",
        watchdog_form(&watchdog),
        monitor_form("rpc-health", "RPC", "/rpc", "ready", true, 30, 10, 3_600),
        monitor_form(
            "federation",
            "Federation",
            "/federation",
            "ready",
            true,
            120,
            30,
            7_200,
        )
    );
    let ids = element_ids(&markup);
    let unique = ids.iter().copied().collect::<HashSet<_>>();
    assert_eq!(ids.len(), unique.len(), "duplicate ids in {markup}");
    for expected in [
        "watchdog-enabled",
        "rpc-health-enabled",
        "rpc-health-interval",
        "federation-enabled",
        "federation-interval",
    ] {
        assert!(unique.contains(expected), "missing {expected} in {markup}");
    }
}

fn element_ids(markup: &str) -> Vec<&str> {
    markup
        .split(r#" id=""#)
        .skip(1)
        .filter_map(|tail| tail.split('"').next())
        .collect()
}
