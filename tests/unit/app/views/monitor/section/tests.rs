use super::MonitorSection;

#[test]
fn every_section_carries_a_label() {
    for section in MonitorSection::ALL {
        assert!(!section.label().is_empty());
    }
}

#[test]
fn sections_round_trip_through_their_index() {
    for (index, section) in MonitorSection::ALL.iter().enumerate() {
        assert_eq!(*section as usize, index);
    }
}

#[test]
fn every_section_round_trips_through_its_persist_key() {
    for section in MonitorSection::ALL {
        assert!(!section.persist_key().is_empty());
        assert_eq!(
            MonitorSection::from_persist_key(section.persist_key()),
            Some(section),
        );
    }
    assert_eq!(MonitorSection::from_persist_key("not-a-section"), None);
}

#[test]
fn persist_keys_are_unique() {
    let mut keys: Vec<_> = MonitorSection::ALL
        .iter()
        .map(|s| s.persist_key())
        .collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), MonitorSection::ALL.len());
}
