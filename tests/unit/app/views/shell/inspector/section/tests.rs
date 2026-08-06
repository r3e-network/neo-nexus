use super::InspectorSection;

#[test]
fn every_section_round_trips_through_its_index() {
    for section in InspectorSection::ALL {
        assert_eq!(InspectorSection::from_index(section.index()), section);
    }
}

#[test]
fn indices_are_dense_and_ordered() {
    for (position, section) in InspectorSection::ALL.iter().enumerate() {
        assert_eq!(section.index(), position);
    }
}

#[test]
fn an_out_of_range_index_falls_back_to_the_default_section() {
    assert_eq!(
        InspectorSection::from_index(InspectorSection::ALL.len()),
        InspectorSection::default()
    );
    assert_eq!(InspectorSection::default(), InspectorSection::Overview);
}

#[test]
fn labels_are_distinct_and_non_empty() {
    let mut labels: Vec<&str> = InspectorSection::ALL
        .iter()
        .map(|section| section.label())
        .collect();
    assert!(labels.iter().all(|label| !label.is_empty()));
    labels.sort_unstable();
    let count = labels.len();
    labels.dedup();
    assert_eq!(labels.len(), count, "section labels must be distinct");
}
