use super::*;

/// Every stage must be reachable from `ALL`, or a section would exist in the
/// type system and never appear as a tab.
#[test]
fn every_stage_is_listed_and_distinct() {
    assert_eq!(PrivateNetworkSection::ALL.len(), 3);
    for (index, section) in PrivateNetworkSection::ALL.into_iter().enumerate() {
        assert_eq!(
            PrivateNetworkSection::ALL
                .into_iter()
                .position(|other| other == section),
            Some(index),
            "{section:?} appears twice",
        );
    }
}

/// The persisted key is what survives a restart, so it must round-trip and must
/// not collide with a sibling's.
#[test]
fn persist_keys_round_trip_and_are_unique() {
    let mut keys = Vec::new();
    for section in PrivateNetworkSection::ALL {
        let key = section.persist_key();
        assert_eq!(
            PrivateNetworkSection::from_persist_key(key),
            Some(section),
            "{key} does not round-trip",
        );
        assert!(!keys.contains(&key), "{key} is used twice");
        keys.push(key);
    }
}

#[test]
fn an_unknown_key_is_rejected_rather_than_defaulted() {
    for unknown in ["", "presets", "PLAN", "signer", "deployment"] {
        assert_eq!(
            PrivateNetworkSection::from_persist_key(unknown),
            None,
            "{unknown}",
        );
    }
}

/// The tabs read left to right in the order the work happens: decide the
/// topology, supply the keys it needs, then deploy it. A reader who follows the
/// tabs in order should not have to go back.
#[test]
fn the_stages_are_ordered_the_way_the_work_happens() {
    assert_eq!(
        PrivateNetworkSection::ALL,
        [
            PrivateNetworkSection::Plan,
            PrivateNetworkSection::Signers,
            PrivateNetworkSection::Deploy,
        ],
    );
}

#[test]
fn every_label_is_operator_facing() {
    for section in PrivateNetworkSection::ALL {
        assert!(!section.label().is_empty());
        assert!(
            !section.label().contains('_'),
            "{:?} shows a persistence key, not a label",
            section,
        );
    }
}
