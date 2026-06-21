use crate::events::{EventKind, EventSeverity, RuntimeEvent, RuntimeEventFilter};

use super::EventJournalReport;

#[test]
fn event_journal_report_redacts_sensitive_messages() {
    let filter = RuntimeEventFilter::new(None, "", 10);
    let report = EventJournalReport::from_events(
        "neonexus.db",
        vec![RuntimeEvent {
            id: 7,
            occurred_at_unix: 1_800_000_001,
            node_id: Some("node-1".to_string()),
            node_name: Some("validator".to_string()),
            kind: EventKind::NodeStartFailed,
            severity: EventSeverity::Critical,
            message: "launch failed Authorization: Bearer journal-token api_key:abc123 seed=raw"
                .to_string(),
        }],
        1,
        &filter,
        "test",
        1_800_000_002,
    );

    let message = &report.events[0].message;
    assert!(message.contains("launch failed"));
    assert!(message.contains("Authorization:<redacted>"));
    assert!(message.contains("api_key:<redacted>"));
    assert!(message.contains("seed=<redacted>"));
    assert!(!message.contains("journal-token"));
    assert!(!message.contains("abc123"));
    assert!(!message.contains("raw"));

    let text = report.to_text();
    assert!(!text.contains("journal-token"));
    assert!(!text.contains("abc123"));
    assert!(!text.contains("seed=raw"));

    let json = report.to_json_text();
    assert!(json.is_ok());
    let Ok(json) = json else {
        return;
    };
    assert!(!json.contains("journal-token"));
    assert!(!json.contains("abc123"));
    assert!(!json.contains("seed=raw"));
}
