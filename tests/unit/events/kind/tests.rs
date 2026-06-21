use super::EventKind;
use std::str::FromStr;

#[test]
fn event_kind_labels_round_trip() {
    for kind in EventKind::ALL.iter().copied() {
        assert!(matches!(EventKind::from_str(kind.label()), Ok(parsed) if parsed == kind));
        assert_eq!(kind.to_string(), kind.label());
    }
}
