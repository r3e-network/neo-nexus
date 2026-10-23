use std::{collections::HashSet, time::Duration};

use super::*;

#[test]
fn settings_forms_assign_document_unique_control_ids() {
    let watchdog = RestartPolicy::new(3, Duration::from_secs(2), Duration::from_secs(30));
    let markup = format!(
        "{}{}{}",
        watchdog_form(&watchdog, None),
        monitor_form(
            "rpc-health",
            "RPC",
            "/rpc",
            "ready",
            true,
            30,
            10,
            3_600,
            None
        ),
        monitor_form(
            "federation",
            "Federation",
            "/federation",
            "ready",
            true,
            120,
            30,
            7_200,
            None,
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

/// The banner above a redrawn form: how many fields, and what the domain said
/// when the failure belongs to none of them.
///
/// The second half is the one that went missing. `summary` counts named fields
/// only, so a whole-policy refusal produced an empty string and the page came
/// back with the operator's input intact and **nothing saying why** — the form
/// looked as if the save had simply not happened.
#[test]
fn the_form_banner_counts_fields_and_quotes_what_belongs_to_none() {
    use crate::web::form_state::{banner, FieldErrors, GENERAL};

    assert_eq!(
        banner(&FieldErrors::new()),
        "",
        "a form with nothing wrong carries no banner"
    );

    let mut one = FieldErrors::new();
    one.insert(
        "interval_seconds",
        "interval must be a whole number".to_string(),
    );
    let markup = banner(&one);
    assert!(markup.contains("One field needs attention."), "{markup}");
    // A named field's message belongs beside that field, drawn by the control
    // itself. Repeating it here would make the page state the same complaint
    // twice, and the second copy would be nowhere near the input it is about.
    assert!(
        !markup.contains("interval must be a whole number"),
        "the banner repeated a message that belongs beside its field: {markup}"
    );

    let mut two = FieldErrors::new();
    two.insert("base_delay_seconds", "one".to_string());
    two.insert("max_delay_seconds", "two".to_string());
    let markup = banner(&two);
    assert!(markup.contains("2 fields need attention."), "{markup}");

    // A refusal that belongs to no field is quoted rather than counted: "one
    // field needs attention" over it would name a control that is not wrong.
    let mut general = FieldErrors::new();
    general.insert(GENERAL, "the policy is inconsistent".to_string());
    let markup = banner(&general);
    assert!(markup.contains("the policy is inconsistent"), "{markup}");
    assert!(
        !markup.contains("needs attention"),
        "a whole-policy refusal must not be counted as a field: {markup}"
    );
}
