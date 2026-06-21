use super::*;

#[test]
fn suggests_closest_option_for_a_typo() {
    assert_eq!(suggest_option("--rpc-helth"), Some("--rpc-health"));
    assert_eq!(
        suggest_option("--validate-walet"),
        Some("--validate-wallet")
    );
    assert_eq!(suggest_option("--verison"), Some("--version"));
    assert_eq!(suggest_option("--source-qualtiy"), Some("--source-quality"));
}

#[test]
fn offers_no_suggestion_for_unrelated_input() {
    assert_eq!(suggest_option("--frobnicate"), None);
    assert_eq!(suggest_option("--xyz"), None);
    assert_eq!(suggest_option("--completely-made-up-flag"), None);
}

#[test]
fn every_known_option_is_its_own_closest_match() {
    for option in KNOWN_OPTIONS {
        assert_eq!(
            suggest_option(option),
            Some(*option),
            "exact option should match itself: {option}"
        );
    }
}

#[test]
fn known_options_are_all_recognized_by_the_dispatcher() {
    for option in KNOWN_OPTIONS {
        if let Err(error) = crate::cli::action_from_args(["neo-nexus", option]) {
            assert!(
                !error.to_string().contains("unsupported NeoNexus option"),
                "{option} is listed as known but the dispatcher rejects it as unsupported"
            );
        }
    }
}
