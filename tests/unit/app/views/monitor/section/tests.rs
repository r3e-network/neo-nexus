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
