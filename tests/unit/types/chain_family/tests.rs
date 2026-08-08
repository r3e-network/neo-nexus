use super::*;

/// Both families must be reachable from `ALL`, or a chain family would exist in
/// the type system and never appear in a picker.
#[test]
fn every_family_is_listed_and_distinct() {
    assert_eq!(ChainFamily::ALL.len(), 2);
    assert_ne!(ChainFamily::ALL[0], ChainFamily::ALL[1]);
}

#[test]
fn slugs_round_trip_and_labels_are_operator_facing() {
    for family in ChainFamily::ALL {
        assert_eq!(ChainFamily::from_slug(family.slug()), Some(family));
        assert!(!family.label().is_empty());
        assert!(!family.slug().contains(' '), "a slug is url/db safe");
    }
    assert_eq!(ChainFamily::NeoN3.label(), "Neo N3");
    assert_eq!(ChainFamily::NeoX.label(), "Neo X");
}

#[test]
fn an_unknown_slug_is_rejected_rather_than_defaulted() {
    for unknown in ["", "neo", "neo-n4", "NEO-N3", "ethereum"] {
        assert_eq!(ChainFamily::from_slug(unknown), None, "{unknown}");
    }
}

/// Plugins are a Neo N3 concept: Neo X clients load no assemblies, so any
/// plugin surface must be hidden rather than shown empty.
/// A private network template writes a Neo N3 committee roster. Neo X takes
/// its validators from a genesis allocation NeoNexus does not generate, so
/// planning one would produce a fleet that never reaches consensus.
#[test]
fn only_neo_n3_can_be_planned_from_a_committee_template() {
    assert!(ChainFamily::NeoN3.has_committee_templates());
    assert!(!ChainFamily::NeoX.has_committee_templates());
}

#[test]
fn only_neo_n3_has_plugins_and_only_neo_x_is_evm() {
    assert!(ChainFamily::NeoN3.has_plugins());
    assert!(!ChainFamily::NeoX.has_plugins());
    assert!(ChainFamily::NeoX.is_evm());
    assert!(!ChainFamily::NeoN3.is_evm());
}
