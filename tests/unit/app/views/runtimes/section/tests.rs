use super::RuntimesSection;

#[test]
fn every_section_carries_a_label() {
    for section in RuntimesSection::ALL {
        assert!(!section.label().is_empty());
    }
}

#[test]
fn sections_round_trip_through_their_index() {
    for (index, section) in RuntimesSection::ALL.iter().enumerate() {
        assert_eq!(*section as usize, index);
    }
}

#[test]
fn every_section_round_trips_through_its_persist_key() {
    for section in RuntimesSection::ALL {
        assert!(!section.persist_key().is_empty());
        assert_eq!(
            RuntimesSection::from_persist_key(section.persist_key()),
            Some(section),
        );
    }
    assert_eq!(RuntimesSection::from_persist_key("not-a-section"), None);
}

#[test]
fn persist_keys_are_unique() {
    let mut keys: Vec<_> = RuntimesSection::ALL
        .iter()
        .map(|s| s.persist_key())
        .collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), RuntimesSection::ALL.len());
}
